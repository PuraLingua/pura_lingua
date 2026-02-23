use std::mem::DropGuard;

use stdlib_header::CoreTypeId;

use crate::{
    stdlib::System_InvalidEnumException_MethodId, type_system::class::Class,
    value::managed_reference::ManagedReference, virtual_machine::cpu::CPU,
};

pub struct ExceptionManager {
    exception: ManagedReference<Class>,
}

impl ExceptionManager {
    pub const fn new() -> Self {
        Self {
            exception: ManagedReference::null(),
        }
    }
    pub fn set(&mut self, exception: ManagedReference<Class>) {
        self.exception = exception;
    }
    pub fn clear(&mut self) {
        self.exception.data = None;
    }
    pub fn has_exception(&self) -> bool {
        !self.exception.is_null()
    }
    pub fn get_exception(&self) -> &ManagedReference<Class> {
        &self.exception
    }
}

#[repr(transparent)]
pub struct ThrowHelper(CPU);

impl ThrowHelper {
    pub fn invalid_enum(&self, enum_name: &str, message: &str) -> bool {
        let destroyer = |mut x: ManagedReference<Class>| {
            x.destroy(&self.0);
        };
        let enum_name = DropGuard::new(ManagedReference::new_string(&self.0, enum_name), destroyer);
        let message = DropGuard::new(ManagedReference::new_string(&self.0, message), destroyer);
        self.0
            .throw_exception(
                match self.0.new_object(
                    &self
                        .0
                        .vm_ref()
                        .assembly_manager()
                        .get_core_type(CoreTypeId::System_InvalidEnumException)
                        .into(),
                    &System_InvalidEnumException_MethodId::Constructor_String_String.into(),
                    &[
                        (&*enum_name as *const ManagedReference<Class>)
                            .cast_mut()
                            .cast(),
                        (&*message as *const ManagedReference<Class>)
                            .cast_mut()
                            .cast(),
                    ],
                ) {
                    None => return false,
                    Some(exception) => exception,
                },
            )
            .unwrap();
        true
    }

    #[cfg(windows)]
    pub fn current_win32(&self) -> bool {
        unsafe { self.win32(windows::Win32::Foundation::GetLastError().0 as i32) }
    }
    #[cfg(windows)]
    pub fn win32(&self, mut code: i32) -> bool {
        use crate::stdlib::System_Win32Exception_MethodId;

        self.0
            .throw_exception(
                match self.0.new_object(
                    &self
                        .0
                        .vm_ref()
                        .assembly_manager()
                        .get_core_type(CoreTypeId::System_Win32Exception)
                        .into(),
                    &System_Win32Exception_MethodId::Constructor_I32.into(),
                    &[(&raw mut code).cast()],
                ) {
                    None => return false,
                    Some(exception) => exception,
                },
            )
            .unwrap();
        true
    }

    has_errno! {
        pub fn current_errno(&self) -> bool {
            let errno = unsafe { *errno_location() };
            self.errno(errno)
        }
    }
    #[cfg(unix)]
    pub fn errno(&self, mut code: i32) -> bool {
        use crate::stdlib::System_ErrnoException_MethodId;

        self.0
            .throw_exception(
                match self.0.new_object(
                    &self
                        .0
                        .vm_ref()
                        .assembly_manager()
                        .get_core_type(CoreTypeId::System_ErrnoException)
                        .into(),
                    &System_ErrnoException_MethodId::Constructor_I32.into(),
                    &[(&raw mut code).cast()],
                ) {
                    None => return false,
                    Some(exception) => exception,
                },
            )
            .unwrap();
        true
    }

    #[cfg(unix)]
    // cSpell:disable-next-line
    pub fn current_dlerror(&self) -> bool {
        // cSpell:disable-next-line
        unsafe { self.dlerror(libc::dlerror()) }
    }
    #[cfg(unix)]
    // cSpell:disable-next-line
    pub fn dlerror(&self, message: *mut libc::c_char) -> bool {
        use crate::stdlib::System_Exception_MethodId;

        let destroyer = |mut x: ManagedReference<Class>| {
            x.destroy(&self.0);
        };

        let message = DropGuard::new(
            ManagedReference::new_string(
                &self.0,
                &unsafe { std::ffi::CString::from_raw(message) }
                    .into_string()
                    .unwrap(),
            ),
            destroyer,
        );

        self.0
            .throw_exception(
                match self.0.new_object(
                    &self
                        .0
                        .vm_ref()
                        .assembly_manager()
                        .get_core_type(CoreTypeId::System_DlErrorException)
                        .into(),
                    System_Exception_MethodId::Constructor_String as u32,
                    &[(&*message as *const ManagedReference<Class>)
                        .cast_mut()
                        .cast()],
                ) {
                    None => return false,
                    Some(exception) => exception,
                },
            )
            .unwrap();
        true
    }
}

impl CPU {
    pub fn throw_helper<'a>(&'a self) -> &'a ThrowHelper {
        unsafe { &*(self as *const Self as *const ThrowHelper) }
    }
}

mod helpers {
    use std::sync::{LockResult, PoisonError, RwLockReadGuard, RwLockWriteGuard};

    use crate::{
        type_system::class::Class, value::managed_reference::ManagedReference,
        virtual_machine::cpu::CPU,
    };

    use super::ExceptionManager;

    impl CPU {
        pub fn exception_manager(&self) -> LockResult<RwLockReadGuard<'_, ExceptionManager>> {
            self.exception_manager.read()
        }
        pub fn exception_manager_mut(&self) -> LockResult<RwLockWriteGuard<'_, ExceptionManager>> {
            self.exception_manager.write()
        }
        pub fn has_exception(
            &self,
        ) -> Result<bool, PoisonError<RwLockReadGuard<'_, ExceptionManager>>> {
            self.exception_manager().map(|x| x.has_exception())
        }
        pub fn throw_exception(
            &self,
            exception: ManagedReference<Class>,
        ) -> Result<(), PoisonError<RwLockWriteGuard<'_, ExceptionManager>>> {
            let mut man = self.exception_manager_mut()?;
            man.set(exception);

            Ok(())
        }
        pub fn get_exception(
            &self,
        ) -> Result<ManagedReference<Class>, PoisonError<RwLockReadGuard<'_, ExceptionManager>>>
        {
            self.exception_manager().map(|x| *x.get_exception())
        }
    }
}

// cSpell:disable
crossfig::alias! {
    has_errno: {
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
            fn errno_location() -> *mut libc::c_int;
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
            fn errno_location() -> *mut libc::c_int;
        }
    }
    any(target_os = "solaris", target_os = "illumos") => {
        unsafe extern "C" {
            #[link_name = "___errno"]
            fn errno_location() -> *mut libc::c_int;
        }
    }
    target_os = "haiku" => {
        unsafe extern "C" {
            #[link_name = "_errnop"]
            fn errno_location() -> *mut libc::c_int;
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
            fn errno_location() -> *mut libc::c_int;
        }
    }
    target_os = "aix" => {
        unsafe extern "C" {
            #[link_name = "_Errno"]
            fn errno_location() -> *mut libc::c_int;
        }
    }
    target_os = "nto" => {
        unsafe extern "C" {
            #[link_name = "__get_errno_ptr"]
            fn errno_location() -> *mut libc::c_int;
        }
    }
    _ => {
        crossfig::switch! {
            has_errno => {
                compile_error!("Cfg conflict");
            }
            _ => {
                #[allow(unused)]
                unsafe fn errno_location() -> *mut std::ffi::c_int {
                    std::ptr::null_mut()
                }
            }
        }
    }
}
// cSpell:enable
