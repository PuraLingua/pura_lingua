mod errno {
    // cSpell:disable
    crossfig::alias! {
        pub has_errno: {
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "watchos",
                target_os = "visionos",
                target_os = "freebsd",

                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "android",
                target_os = "espidf",
                target_os = "vxworks",
                target_os = "cygwin",
                target_env = "newlib",

                target_os = "solaris", target_os = "illumos",

                target_os = "haiku",

                target_os = "linux",
                target_os = "hurd",
                target_os = "redox",
                target_os = "dragonfly",
                target_os = "emscripten",

                target_os = "aix",

                target_os = "nto",
            ))]
        }
    }

    // From https://github.com/lambda-fairy/rust-errno/blob/62ef494d8c089610404bcb67637810a46058bc10/src/unix.rs
    cfg_select! {
        any(
            target_os = "macos",
            target_os = "ios",
            target_os = "tvos",
            target_os = "watchos",
            target_os = "visionos",
            target_os = "freebsd"
        ) => {
            unsafe extern "C" {
                #[link_name = "__error"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        any(
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "android",
            target_os = "espidf",
            target_os = "vxworks",
            target_os = "cygwin",
            target_env = "newlib"
        ) => {
            unsafe extern "C" {
                #[link_name = "__errno"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        any(target_os = "solaris", target_os = "illumos") => {
            unsafe extern "C" {
                #[link_name = "___errno"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        target_os = "haiku" => {
            unsafe extern "C" {
                #[link_name = "_errnop"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        any(
            target_os = "linux",
            target_os = "hurd",
            target_os = "redox",
            target_os = "dragonfly",
            target_os = "emscripten",
        ) => {

            unsafe extern "C" {
                #[link_name = "__errno_location"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        target_os = "aix" => {
            unsafe extern "C" {
                #[link_name = "_Errno"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        target_os = "nto" => {
            unsafe extern "C" {
                #[link_name = "__get_errno_ptr"]
                pub fn errno_location() -> *mut libc::c_int;
            }
        }
        _ => {
            crossfig::switch! {
                has_errno => {
                    compile_error!("Cfg conflict");
                }
                _ => {
                    #[allow(unused)]
                    pub unsafe fn errno_location() -> *mut std::ffi::c_int {
                        std::ptr::null_mut()
                    }
                }
            }
        }
    }
    // cSpell:enable
}

#[cfg_attr(windows, expect(unused))]
pub use errno::{errno_location, has_errno};
