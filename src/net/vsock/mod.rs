
pub mod general;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(windows)]
pub mod utils;
#[cfg(windows)]
pub mod windows;

pub use general::{VListener, VStream};

use uuid::Uuid;

/// Whether `err` says this host has no usable vsock transport, rather than
/// that one call failed: no AF_VSOCK, no loopback transport, no platform support.
pub fn is_unavailable(err: &std::io::Error) -> bool {
    if err.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        matches!(
            err.raw_os_error(),
            Some(libc::EAFNOSUPPORT | libc::EADDRNOTAVAIL | libc::ENODEV)
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VsockTarget {
    Cid(u32),
    Guid(Uuid),
}
