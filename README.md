# oguRPChik 🥒

<div align="center">
  <img src="/Unusual-Bananas-0.png" width="350" alt="Ogurpchik Logo">
</div>

RPC between host, VM agents and plugins over vsock / uds / named pipes.
Used in [uniproc](https://github.com/ignat/uniproc).

- capnp-rpc for the protocol (your `.capnp` schema is the contract)
- compio transport: Hyper-V `AF_HYPERV` / Linux vsock, Unix sockets, Windows named pipes, TCP
- pre-RPC handshake on the raw connection: HMAC or signed-process auth with the peer PID taken from the OS (`GetNamedPipeClientProcessId`, `SO_PEERCRED`), never from the wire
- discovery by convention: `\\.\pipe\<app>.<service>`, `$XDG_RUNTIME_DIR/<app>/<service>.sock` (without it, a `<tmp>/<app>-<uid>` directory that must be the caller's own and closed to others), per-service vsock port
- single-threaded, `!Send`, `error-stack` errors

## Hello world

Your schema (compiled with `capnpc` in your own crate):

```capnp
interface Echo {
  ping @0 (msg :Text) -> (reply :Text);
}
```

Server:

```rust
use ogurpchik::auth::handshake::{HandshakeMode, Protocol};
use ogurpchik::endpoint::Endpoint;
use ogurpchik::rpc::SessionAcceptor;

const SCHEMA: Protocol = Protocol::new(0x0123_4567_89ab_cdef, 1, 0, 0);

let endpoint = Endpoint::for_service("myapp", "echo")?;
let listener = endpoint.listen().await?;
let mut acceptor = SessionAcceptor::new(&listener, HandshakeMode::hmac(b"secret".to_vec()), SCHEMA);
loop {
    let session = acceptor.next::<echo_capnp::echo::Client, _>(EchoImpl).await?;
    compio::runtime::spawn(async move { session.wait().await }).detach();
}
```

`SessionAcceptor` runs handshakes side by side, up to `max_pending` at once and
each within `handshake_deadline`, and hands out only sessions that passed; a
client that connects and says nothing costs one slot, not the listener. Serve
each session on its own task. `accept_session`, which handshakes inline, is
deprecated for that reason.

Client:

```rust
use ogurpchik::rpc::connect_session;

let session = connect_session::<echo_capnp::echo::Client, _>(
    &endpoint,
    &HandshakeMode::hmac(b"secret".to_vec()),
    SCHEMA,
    EchoImpl,
).await?;

let mut req = session.remote().ping_request();
req.get().set_msg("hello");
let reply = req.send().promise.await?;
```

`Protocol` is a fixed id plus a semantic version. Use the id of your `.capnp`
file: it never changes. Keep the version by capnp's evolution rules — fields and
methods appended at the end are a minor bump, anything else that touches the
wire is a major one.

The handshake refuses a peer with another id (`HandshakeError::ProtocolMismatch`)
or another major (`HandshakeError::IncompatibleVersion`). A different minor
connects: capnp ignores what it does not know, a method the peer lacks answers
`Unimplemented`, and `session.peer_version()` tells you up front which
additions the peer has.

## Errors

Local errors are `error_stack::Report<C>` over layered contexts
(`EndpointError` → `TransportError` → `HandshakeError` → `RpcError`); match the
top with `current_context()`, reach a layer with `downcast_ref()`.

Only what you opt in crosses the wire — attachments (paths, PIDs, addresses)
stay local, since the peer may be an untrusted plugin. Attach a `WireCode` to
make a failure matchable on the other side:

```rust
use ogurpchik::error::{RpcError, WireCode, WireMessage};

Report::new(RpcError::Handler)
    .attach(WireCode::PermissionDenied)
    .attach(WireMessage("plugin may not read host config".into()))
```

The peer receives `RpcError::Remote { code: Some(WireCode::PermissionDenied), .. }`.
Handlers returning `capnp::Error` directly can use
`WireCode::InvalidArgument.exception("port must be non-zero")`.

On the wire this is the exception text `[ogurpchik:<code>] <message>` — a
non-Rust plugin can produce and parse it. Unknown codes decode to
`WireCode::Unknown` rather than failing, so a newer peer stays compatible.

A received code is a claim by the peer, not a verified fact: use it for
diagnostics and retry decisions, never for authorization.

## License

MIT
