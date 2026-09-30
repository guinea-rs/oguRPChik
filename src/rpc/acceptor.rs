use super::{RpcSession, Side, spawn_session};
use crate::auth::handshake::{HandshakeMode, Protocol, Version, authenticate_server};
use crate::error::{RpcError, TransportError};
use crate::net::{Conn, Listener};
use capnp::capability::{FromClientHook, FromServer};
use error_stack::{Report, ResultExt};
use futures::StreamExt;
use futures::future::{self, Either};
use futures::stream::FuturesUnordered;
use std::future::Future;
use std::pin::{Pin, pin};
use std::time::Duration;

/// Handshakes that may run at once before new connections are dropped.
pub const DEFAULT_MAX_PENDING: usize = 64;

/// Longest a whole handshake may take before its connection is dropped.
pub const DEFAULT_HANDSHAKE_DEADLINE: Duration = Duration::from_secs(10);

type Accepting<'l> = Pin<Box<dyn Future<Output = Result<Conn, Report<TransportError>>> + 'l>>;
type Handshake = Pin<Box<dyn Future<Output = Option<(Conn, Option<Version>)>>>>;

enum Step {
    Accepted(Result<Conn, Report<TransportError>>),
    Settled(Option<(Conn, Option<Version>)>),
}

/// Accepts connections and runs their handshakes concurrently, handing out
/// only sessions that passed. A client that connects and stays silent holds
/// one slot until the deadline, not the listener.
///
/// ```ignore
/// let mut acceptor = SessionAcceptor::new(&listener, mode, protocol);
/// loop {
///     let session = acceptor.next::<my::Client, _>(MyServer).await?;
///     compio::runtime::spawn(serve(session)).detach();
/// }
/// ```
pub struct SessionAcceptor<'l> {
    listener: &'l Listener,
    mode: HandshakeMode,
    protocol: Protocol,
    max_pending: usize,
    deadline: Duration,
    accepting: Option<Accepting<'l>>,
    pending: FuturesUnordered<Handshake>,
}

impl<'l> SessionAcceptor<'l> {
    pub fn new(listener: &'l Listener, mode: HandshakeMode, protocol: Protocol) -> Self {
        Self {
            listener,
            mode,
            protocol,
            max_pending: DEFAULT_MAX_PENDING,
            deadline: DEFAULT_HANDSHAKE_DEADLINE,
            accepting: None,
            pending: FuturesUnordered::new(),
        }
    }

    /// Handshakes allowed at once; a connection arriving past it is dropped.
    pub fn max_pending(mut self, max_pending: usize) -> Self {
        self.max_pending = max_pending.max(1);
        self
    }

    /// Longest one handshake may take, from accept to its last packet.
    pub fn handshake_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    /// Handshakes under way right now.
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// The next session whose handshake passed. Failed handshakes are logged
    /// and skipped; only an error of the listener itself is returned.
    pub async fn next<C, S>(&mut self, local_bootstrap: S) -> crate::error::Result<RpcSession<C>, RpcError>
    where
        C: FromServer<S> + FromClientHook,
        S: 'static,
    {
        loop {
            let step = {
                let listener = self.listener;
                let accepting = self.accepting.get_or_insert_with(|| Box::pin(listener.accept()));
                let pending = &mut self.pending;
                let settle = pin!(async move {
                    if pending.is_empty() {
                        future::pending::<()>().await;
                    }
                    pending.next().await.flatten()
                });
                match future::select(accepting.as_mut(), settle).await {
                    Either::Left((accepted, _)) => Step::Accepted(accepted),
                    Either::Right((settled, _)) => Step::Settled(settled),
                }
            };
            match step {
                Step::Accepted(accepted) => {
                    self.accepting = None;
                    let conn = accepted.change_context(RpcError::Setup)?;
                    if self.pending.len() >= self.max_pending {
                        tracing::warn!(
                            max_pending = self.max_pending,
                            "dropping a connection: too many handshakes under way"
                        );
                        drop(conn);
                        continue;
                    }
                    self.pending.push(self.handshake(conn));
                }
                Step::Settled(Some((conn, peer_version))) => {
                    let mut session = spawn_session(conn, Side::Server, local_bootstrap);
                    session.peer_version = peer_version;
                    return Ok(session);
                }
                Step::Settled(None) => {}
            }
        }
    }

    fn handshake(&self, mut conn: Conn) -> Handshake {
        let (mode, protocol, deadline) = (self.mode.clone(), self.protocol, self.deadline);
        Box::pin(async move {
            match compio::time::timeout(deadline, authenticate_server(&mut conn, &mode, protocol)).await {
                Ok(Ok(peer_version)) => Some((conn, peer_version)),
                Ok(Err(report)) => {
                    tracing::warn!(error = ?report, "handshake failed");
                    None
                }
                Err(_) => {
                    tracing::warn!(?deadline, "handshake did not finish in time");
                    None
                }
            }
        })
    }
}
