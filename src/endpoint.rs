
use crate::error::{EndpointError, Result, TransportError};
use crate::net::vsock::VsockTarget;
use crate::net::{Conn, Listener};
use error_stack::Report;
#[cfg(windows)]
use error_stack::ResultExt;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Longest a single attempt inside [`Endpoint::connect_ready`] may take.
pub const CONNECT_ATTEMPT_LIMIT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum Endpoint {
    Vsock { target: VsockTarget, port: u32 },
    Uds(PathBuf),
    Npipe(String),
    Tcp(SocketAddr),
}

impl Endpoint {
    pub fn for_service(app: &str, service: &str) -> Result<Self, EndpointError> {
        validate_name(app)?;
        validate_name(service)?;

        #[cfg(windows)]
        {
            Ok(Self::Npipe(format!(r"\\.\pipe\{app}.{service}")))
        }
        #[cfg(not(windows))]
        {
            Ok(Self::Uds(runtime_dir(app)?.join(format!("{service}.sock"))))
        }
    }

    pub fn vsock_to_host(port: u32) -> Self {
        Self::Vsock {
            target: VsockTarget::Cid(2),
            port,
        }
    }

    /// The WSL2 VM, or [`EndpointError::WslNotRunning`] when there is none.
    /// Unelevated with several VMs running, [`EndpointError::WslVmAmbiguous`].
    #[cfg(windows)]
    pub fn vsock_to_wsl(port: u32) -> Result<Self, EndpointError> {
        use crate::net::vsock::utils::{WslLookupError, wsl_vmid};
        let vm = match wsl_vmid() {
            Ok(vm) => vm,
            Err(WslLookupError::Ambiguous(count)) => {
                return Err(Report::new(EndpointError::WslVmAmbiguous)
                    .attach(format!("{count} VMs are running; run elevated to tell WSL's apart")));
            }
            Err(WslLookupError::Io(err)) => {
                return Err(Report::new(err).change_context(EndpointError::InvalidVsockTarget));
            }
        }
        .ok_or_else(|| Report::new(EndpointError::WslNotRunning))?;
        Ok(Self::Vsock {
            target: VsockTarget::Guid(vm),
            port,
        })
    }

    #[cfg(windows)]
    #[deprecated(note = "falls back to any VM when WSL is not running; use vsock_to_wsl")]
    pub fn vsock_to_best_vm(port: u32) -> Result<Self, EndpointError> {
        let guid = crate::net::vsock::utils::get_best_vmid()
            .change_context(EndpointError::InvalidVsockTarget)?;
        Ok(Self::Vsock {
            target: VsockTarget::Guid(crate::net::vsock::utils::guid_to_uuid(guid)),
            port,
        })
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Vsock { .. } => "vsock",
            Self::Uds(_) => "uds",
            Self::Npipe(_) => "npipe",
            Self::Tcp(_) => "tcp",
        }
    }

    pub async fn listen(&self) -> Result<Listener, TransportError> {
        match self {
            Self::Tcp(addr) => Listener::bind_tcp(*addr).await,
            Self::Uds(path) => Listener::bind_uds(path).await,
            #[cfg(windows)]
            Self::Npipe(name) => Listener::bind_npipe(name).await,
            #[cfg(not(windows))]
            Self::Npipe(_) => Err(Report::new(TransportError::Bind)
                .attach("named pipes are only available on Windows")),
            Self::Vsock { target, port } => Listener::bind_vsock(*target, *port),
        }
    }

    pub async fn connect(&self) -> Result<Conn, TransportError> {
        match self {
            Self::Tcp(addr) => Conn::connect_tcp(*addr).await,
            Self::Uds(path) => Conn::connect_uds(path).await,
            #[cfg(windows)]
            Self::Npipe(name) => Conn::connect_npipe(name).await,
            #[cfg(not(windows))]
            Self::Npipe(_) => Err(Report::new(TransportError::Connect)
                .attach("named pipes are only available on Windows")),
            Self::Vsock { target, port } => Conn::connect_vsock(*target, *port).await,
        }
    }

    /// Retries [`connect`](Self::connect) until it succeeds or `give_up_after` has passed.
    ///
    /// Each attempt is cut off after at most [`CONNECT_ATTEMPT_LIMIT`]: a Hyper-V
    /// socket connecting to a port nobody listens on yet does not fail until its
    /// own 30-second timeout, so an uncut attempt would outlast the service
    /// starting to listen.
    pub async fn connect_ready(&self, give_up_after: Duration) -> Result<Conn, TransportError> {
        let start = std::time::Instant::now();
        let mut delay = Duration::from_millis(50);
        loop {
            let limit = give_up_after
                .saturating_sub(start.elapsed())
                .min(CONNECT_ATTEMPT_LIMIT);
            let (report, cut_off) = match compio::time::timeout(limit, self.connect()).await {
                Ok(Ok(conn)) => return Ok(conn),
                Ok(Err(report)) => (report, false),
                Err(_) => (
                    Report::new(TransportError::Connect)
                        .attach(format!("connect attempt cut off after {limit:?}")),
                    true,
                ),
            };
            let pause = if cut_off { Duration::ZERO } else { delay };
            if start.elapsed() + pause >= give_up_after {
                return Err(report
                    .attach(format!("endpoint {self}"))
                    .attach("gave up waiting for the service to become ready"));
            }
            if !cut_off {
                compio::time::sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(1));
            }
        }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vsock { target, port } => write!(f, "vsock {target:?}:{port}"),
            Self::Uds(path) => write!(f, "uds {}", path.display()),
            Self::Npipe(name) => write!(f, "npipe {name}"),
            Self::Tcp(addr) => write!(f, "tcp {addr}"),
        }
    }
}

fn validate_name(name: &str) -> Result<(), EndpointError> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(())
    } else {
        Err(Report::new(EndpointError::InvalidServiceName).attach(format!("name: {name:?}")))
    }
}

#[cfg(not(windows))]
fn runtime_dir(app: &str) -> Result<PathBuf, EndpointError> {
    let base = match std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
    {
        Some(xdg) => xdg,
        None => {
            // SAFETY: getuid has no failure mode.
            let uid = unsafe { libc::getuid() };
            let fallback = std::env::temp_dir().join(format!("{app}-{uid}"));
            private_dir(&fallback, uid)?;
            fallback
        }
    };
    let dir = base.join(app);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return Err(Report::new(EndpointError::NoRuntimeDirectory)
            .attach(format!("cannot create {}: {e}", dir.display())));
    }
    Ok(dir)
}

#[cfg(not(windows))]
fn private_dir(dir: &std::path::Path, uid: u32) -> Result<(), EndpointError> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};

    let refuse = |why: String| {
        Err(Report::new(EndpointError::NoRuntimeDirectory).attach(format!("{}: {why}", dir.display())))
    };
    match std::fs::DirBuilder::new().mode(0o700).create(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return refuse(format!("cannot create: {e}")),
    }
    let metadata = match std::fs::symlink_metadata(dir) {
        Ok(metadata) => metadata,
        Err(e) => return refuse(format!("cannot inspect: {e}")),
    };
    if !metadata.file_type().is_dir() {
        return refuse("not a directory".into());
    }
    if metadata.uid() != uid {
        return refuse(format!("owned by uid {}, not {uid}", metadata.uid()));
    }
    if metadata.mode() & 0o077 != 0 {
        return refuse(format!("mode {:o} lets others in", metadata.mode() & 0o777));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_name_validation() {
        assert!(Endpoint::for_service("myapp", "metrics-v2").is_ok());
        for bad in ["", "a/b", "a\\b", "a b", "a.b", "a:b", &"x".repeat(65)] {
            let report = Endpoint::for_service("myapp", bad)
                .expect_err("invalid service name must be rejected");
            assert!(matches!(
                report.current_context(),
                EndpointError::InvalidServiceName
            ));
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_convention_is_named_pipe() {
        match Endpoint::for_service("myapp", "metrics").unwrap() {
            Endpoint::Npipe(name) => assert_eq!(name, r"\\.\pipe\myapp.metrics"),
            other => panic!("expected npipe, got {other}"),
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_convention_is_uds_in_runtime_dir() {
        match Endpoint::for_service("myapp", "metrics").unwrap() {
            Endpoint::Uds(path) => assert!(path.ends_with("myapp/metrics.sock")),
            other => panic!("expected uds, got {other}"),
        }
    }

    #[cfg(not(windows))]
    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ogurpchik-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[cfg(not(windows))]
    #[test]
    fn private_dir_is_created_closed_and_accepted_again() {
        use std::os::unix::fs::MetadataExt;
        let dir = scratch("private");
        let uid = unsafe { libc::getuid() };
        private_dir(&dir, uid).expect("fresh directory");
        assert_eq!(std::fs::metadata(&dir).unwrap().mode() & 0o077, 0);
        private_dir(&dir, uid).expect("own private directory again");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn private_dir_refuses_what_others_can_enter() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("open");
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o777)).unwrap();
        let uid = unsafe { libc::getuid() };
        let report = private_dir(&dir, uid).expect_err("a world-writable directory");
        assert!(matches!(report.current_context(), EndpointError::NoRuntimeDirectory));
        let report = private_dir(&dir, uid.wrapping_add(1)).expect_err("someone else's directory");
        assert!(matches!(report.current_context(), EndpointError::NoRuntimeDirectory));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(not(windows))]
    #[test]
    fn private_dir_refuses_a_symlink() {
        let target = scratch("target");
        let link = scratch("link");
        std::fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let uid = unsafe { libc::getuid() };
        assert!(private_dir(&link, uid).is_err());
        std::fs::remove_file(&link).unwrap();
        std::fs::remove_dir_all(&target).unwrap();
    }

    #[compio::test]
    async fn tcp_endpoint_listen_connect_roundtrip() {
        let placeholder = Endpoint::Tcp("127.0.0.1:0".parse().unwrap());
        let listener = placeholder.listen().await.expect("listen failed");
        let Listener::Tcp(inner) = &listener else {
            unreachable!()
        };
        let endpoint = Endpoint::Tcp(inner.local_addr().expect("local_addr failed"));

        let (server, client) = futures::try_join!(listener.accept(), endpoint.connect())
            .expect("join failed");
        assert_eq!(server.kind(), "tcp");
        assert_eq!(client.kind(), "tcp");
    }

    #[cfg(windows)]
    #[compio::test]
    async fn conventional_npipe_endpoint_roundtrip() {
        let endpoint =
            Endpoint::for_service("ogurpchik-test", &format!("ep-{}", std::process::id()))
                .expect("convention failed");
        let listener = endpoint.listen().await.expect("listen failed");
        let (server, client) = futures::try_join!(listener.accept(), endpoint.connect())
            .expect("join failed");
        assert_eq!(server.kind(), "npipe");
        assert_eq!(client.kind(), "npipe");
    }

    #[compio::test]
    async fn connect_ready_succeeds_once_service_appears() {
        let placeholder = Endpoint::Tcp("127.0.0.1:0".parse().unwrap());
        let early = placeholder.listen().await.expect("listen failed");
        let Listener::Tcp(inner) = &early else {
            unreachable!()
        };
        let endpoint = Endpoint::Tcp(inner.local_addr().expect("local_addr failed"));
        drop(early);

        let listener_task = compio::runtime::spawn({
            let endpoint = endpoint.clone();
            async move {
                compio::time::sleep(Duration::from_millis(200)).await;
                endpoint.listen().await.expect("delayed listen failed")
            }
        });

        let conn = endpoint
            .connect_ready(Duration::from_secs(5))
            .await
            .expect("connect_ready gave up too early");
        drop(conn);
        let listener = listener_task.await.unwrap();
        let server = listener.accept().await.expect("accept failed");
        assert_eq!(server.kind(), "tcp");
    }

    #[compio::test]
    async fn connect_ready_keeps_to_its_budget_when_an_attempt_hangs() {
        let unroutable = Endpoint::Tcp("192.0.2.1:9".parse().unwrap());
        let start = std::time::Instant::now();
        let result = unroutable.connect_ready(Duration::from_millis(1500)).await;
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_millis(2500), "{:?}", start.elapsed());
    }
}
