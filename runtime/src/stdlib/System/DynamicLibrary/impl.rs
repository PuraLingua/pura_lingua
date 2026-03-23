#[cfg_attr(windows, path = "./impl/windows.rs")]
#[cfg_attr(unix, path = "./impl/unix.rs")]
mod os_specific;

pub(super) use os_specific::*;
