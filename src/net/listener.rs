use crate::error::TransportError;
use crate::net::Conn;
use crate::net::vsock::{VListener, VsockTarget};
use compio::net::{TcpListener, UnixListener};
use error_stack::{Report, ResultExt};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use crate::net::npipe::NamedPipeAcceptor;

pub enum Listener {
    Tcp(TcpListener),
    Uds {
        listener: UnixListener,
        path: PathBuf,
        identity: Option<(u64, u64)>,
    },
    #[cfg(windows)]
    Npipe(NamedPipeAcceptor),
    Vsock(VListener),
}

impl Listener {
    pub async fn bind_tcp(addr: SocketAddr) -> Result<Self, Report<TransportError>> {
        TcpListener::bind(addr)
            .await
            .map(Self::Tcp)
            .change_context(TransportError::Bind)
            .attach(format!("tcp {addr}"))
    }

    /// Binds `path`, first removing a socket file there that nobody listens on.
    /// A live socket or any other file at `path` is left alone and refused.
    pub async fn bind_uds(path: &Path) -> Result<Self, Report<TransportError>> {
        clear_stale_socket(path)
            .await
            .change_context(TransportError::Bind)
            .attach(format!("uds {}", path.display()))?;
        UnixListener::bind(path)
            .await
            .map(|listener| Self::Uds {
                listener,
                path: path.to_path_buf(),
                identity: file_identity(path),
            })
            .change_context(TransportError::Bind)
            .attach(format!("uds {}", path.display()))
    }

    #[cfg(windows)]
    pub async fn bind_npipe(name: &str) -> Result<Self, Report<TransportError>> {
        NamedPipeAcceptor::bind(name)
            .await
            .map(Self::Npipe)
            .change_context(TransportError::Bind)
            .attach(format!("npipe {name}"))
    }

    pub fn bind_vsock(target: VsockTarget, port: u32) -> Result<Self, Report<TransportError>> {
        VListener::bind(target, port)
            .map(Self::Vsock)
            .change_context(TransportError::Bind)
            .attach(format!("vsock port {port}"))
    }

    pub fn bind_vsock_loopback(port: u32) -> Result<Self, Report<TransportError>> {
        VListener::bind_loopback(port)
            .map(Self::Vsock)
            .change_context(TransportError::Bind)
            .attach(format!("vsock loopback port {port}"))
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Tcp(_) => "tcp",
            Self::Uds { .. } => "uds",
            #[cfg(windows)]
            Self::Npipe(_) => "npipe",
            Self::Vsock(_) => "vsock",
        }
    }

    pub async fn accept(&self) -> Result<Conn, Report<TransportError>> {
        match self {
            Self::Tcp(l) => l
                .accept()
                .await
                .map(|(s, _)| Conn::Tcp(s))
                .change_context(TransportError::Accept),
            Self::Uds { listener, .. } => listener
                .accept()
                .await
                .map(|(s, _)| Conn::Uds(s))
                .change_context(TransportError::Accept),
            #[cfg(windows)]
            Self::Npipe(l) => l
                .accept()
                .await
                .map(Conn::Npipe)
                .change_context(TransportError::Accept),
            Self::Vsock(l) => l
                .accept()
                .await
                .map(|(s, _)| Conn::Vsock(s))
                .change_context(TransportError::Accept),
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Self::Uds { path, identity, .. } = self
            && (cfg!(windows) || (identity.is_some() && file_identity(path) == *identity))
        {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(unix)]
fn file_identity(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(path).ok().map(|m| (m.dev(), m.ino()))
}

#[cfg(windows)]
fn file_identity(_path: &Path) -> Option<(u64, u64)> {
    None
}

#[cfg(unix)]
fn is_socket_file(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::FileTypeExt;
    metadata.file_type().is_socket()
}

#[cfg(windows)]
fn is_socket_file(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    !metadata.file_type().is_symlink() && metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

async fn clear_stale_socket(path: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    if !is_socket_file(&metadata) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            "the path holds something other than a socket; not removing it",
        ));
    }
    match compio::net::UnixStream::connect(path).await {
        Ok(_) => Err(Error::new(ErrorKind::AddrInUse, "a server is listening on this socket")),
        Err(err) if err.kind() == ErrorKind::ConnectionRefused => std::fs::remove_file(path),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compio::BufResult;
    use compio::io::{AsyncReadExt, AsyncWriteExt};

    async fn ping_pong(mut server: Conn, mut client: Conn) {
        let BufResult(res, _) = client.write_all(b"ping").await;
        res.expect("client write failed");

        let buf = vec![0u8; 4];
        let BufResult(res, buf) = server.read_exact(buf).await;
        res.expect("server read failed");
        assert_eq!(&buf, b"ping");

        let BufResult(res, _) = server.write_all(b"pong").await;
        res.expect("server write failed");

        let buf = vec![0u8; 4];
        let BufResult(res, buf) = client.read_exact(buf).await;
        res.expect("client read failed");
        assert_eq!(&buf, b"pong");
    }

    #[compio::test]
    async fn tcp_ping_pong_through_conn_enum() {
        let listener = Listener::bind_tcp("127.0.0.1:0".parse().unwrap())
            .await
            .expect("bind failed");
        let Listener::Tcp(inner) = &listener else {
            unreachable!()
        };
        let addr = inner.local_addr().expect("local_addr failed");

        let accept = listener.accept();
        let connect = Conn::connect_tcp(addr);
        let (server, client) = futures::try_join!(accept, connect).expect("join failed");

        assert_eq!(server.kind(), "tcp");
        assert_eq!(client.kind(), "tcp");
        ping_pong(server, client).await;
    }

    #[compio::test]
    async fn uds_ping_pong_through_conn_enum() {
        let path = std::env::temp_dir().join(format!("ogurpchik-net-test-{}.sock", std::process::id()));

        let listener = Listener::bind_uds(&path).await.expect("bind failed");
        let accept = listener.accept();
        let connect = Conn::connect_uds(&path);
        let (server, client) = futures::try_join!(accept, connect).expect("join failed");

        ping_pong(server, client).await;

        drop(listener);
        assert!(!path.exists(), "uds socket file should be cleaned up on drop");
    }

    fn uds_path(tag: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("ogurpchik-net-test-{tag}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[compio::test]
    async fn uds_rebind_over_a_stale_socket_succeeds() {
        let path = uds_path("stale");
        drop(UnixListener::bind(&path).await.expect("raw bind failed"));
        assert!(path.exists(), "a dropped raw listener leaves its socket file behind");

        let _listener = Listener::bind_uds(&path)
            .await
            .expect("bind should clear the stale socket and succeed");
    }

    #[compio::test]
    async fn uds_bind_leaves_a_regular_file_alone() {
        let path = uds_path("regular");
        std::fs::write(&path, b"not a socket").expect("write failed");

        assert!(Listener::bind_uds(&path).await.is_err());
        assert_eq!(std::fs::read(&path).expect("the file is still there"), b"not a socket");
        std::fs::remove_file(&path).unwrap();
    }

    #[compio::test]
    async fn uds_bind_refuses_a_live_socket_and_keeps_it() {
        let path = uds_path("live");
        let first = Listener::bind_uds(&path).await.expect("first bind failed");

        assert!(Listener::bind_uds(&path).await.is_err());
        drop(first.accept().await.expect("the second bind's probe"));
        let (server, client) = futures::try_join!(first.accept(), Conn::connect_uds(&path))
            .expect("the first listener still serves its path");
        ping_pong(server, client).await;
    }

    #[cfg(windows)]
    #[compio::test]
    async fn npipe_ping_pong_through_conn_enum() {
        let name = format!("ogurpchik-net-conn-test-{}", std::process::id());

        let listener = Listener::bind_npipe(&name).await.expect("bind failed");
        let accept = listener.accept();
        let connect = Conn::connect_npipe(&name);
        let (server, client) = futures::try_join!(accept, connect).expect("join failed");

        assert_eq!(server.kind(), "npipe");
        ping_pong(server, client).await;
    }

    #[compio::test]
    async fn vsock_ping_pong_through_conn_enum() {
        const PORT: u32 = 22345;
        let listener = match Listener::bind_vsock_loopback(PORT) {
            Ok(listener) => listener,
            Err(report)
                if report
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(crate::net::vsock::is_unavailable) =>
            {
                eprintln!("skipped: no vsock loopback on this host");
                return;
            }
            Err(report) => panic!("bind failed: {report:?}"),
        };

        let accept = listener.accept();
        let connect = Conn::connect_vsock_loopback(PORT);
        let (server, client) = futures::try_join!(accept, connect).expect("join failed");

        assert_eq!(server.kind(), "vsock");
        ping_pong(server, client).await;
    }
}
