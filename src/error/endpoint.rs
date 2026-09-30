use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum EndpointError {
    InvalidServiceName,
    InvalidVsockTarget,
    UnsupportedOnPlatform,
    NoRuntimeDirectory,
    WslNotRunning,
    WslVmAmbiguous,
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidServiceName => f.write_str("invalid service name"),
            Self::InvalidVsockTarget => f.write_str("invalid vsock target"),
            Self::UnsupportedOnPlatform => {
                f.write_str("transport not supported on this platform")
            }
            Self::NoRuntimeDirectory => f.write_str("no usable runtime directory"),
            Self::WslNotRunning => f.write_str("no running WSL VM"),
            Self::WslVmAmbiguous => f.write_str("cannot tell the WSL VM from the others"),
        }
    }
}

impl core::error::Error for EndpointError {}
