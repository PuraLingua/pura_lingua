use stdlib_header::CoreTypeId;

use crate::{
    type_system::class::Class, value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
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
    pub fn alloc(&mut self) -> bool {
        let exception = match self.0.new_object(
            &self
                .0
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_AllocException)
                .into(),
            &stdlib_header::MethodId!(AllocException::Constructor).into(),
            &[],
        ) {
            None => return false,
            Some(exception) => exception,
        };
        self.0.throw_exception(exception);

        true
    }
    pub fn invalid_enum(&mut self, enum_name: &str, message: &str) -> bool {
        let mut enum_name = ManagedReference::new_string(&mut self.0, enum_name);
        let mut message = ManagedReference::new_string(&mut self.0, message);

        let exception = match self.0.new_object(
            &self
                .0
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_InvalidEnumException)
                .into(),
            &stdlib_header::MethodId!(InvalidEnumException::Constructor_String_String).into(),
            &[(&raw mut enum_name).cast(), (&raw mut message).cast()],
        ) {
            None => return false,
            Some(exception) => exception,
        };
        self.0.throw_exception(exception);

        true
    }

    #[cfg(windows)]
    pub fn current_win32(&mut self) -> bool {
        unsafe { self.win32(windows::Win32::Foundation::GetLastError().0 as i32) }
    }
    #[cfg(windows)]
    pub fn win32(&mut self, mut code: i32) -> bool {
        let exception = match self.0.new_object(
            &self
                .0
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_Win32Exception)
                .into(),
            &stdlib_header::MethodId!(Win32Exception::Constructor_I32).into(),
            &[(&raw mut code).cast()],
        ) {
            None => return false,
            Some(exception) => exception,
        };

        self.0.throw_exception(exception);
        true
    }

    crate::c_ffi::has_errno! {
        pub fn current_errno(&mut self) -> bool {
            let errno = unsafe { *crate::c_ffi::errno_location() };
            self.errno(errno)
        }
    }
    #[doc(cfg(unix))]
    #[cfg(unix)]
    pub fn errno(&mut self, mut code: i32) -> bool {
        let exception = match self.0.new_object(
            &self
                .0
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_ErrnoException)
                .into(),
            &stdlib_header::MethodId!(ErrnoException::Constructor_I32).into(),
            &[(&raw mut code).cast()],
        ) {
            None => return false,
            Some(exception) => exception,
        };

        self.0.throw_exception(exception);
        true
    }

    #[doc(cfg(unix))]
    #[cfg(unix)]
    // cSpell:disable-next-line
    pub fn current_dlerror(&mut self) -> bool {
        // cSpell:disable-next-line
        unsafe { self.dlerror(libc::dlerror()) }
    }
    #[doc(cfg(unix))]
    #[cfg(unix)]
    // cSpell:disable-next-line
    pub fn dlerror(&mut self, message: *mut libc::c_char) -> bool {
        let message = ManagedReference::new_string(
            &mut self.0,
            &unsafe { std::ffi::CString::from_raw(message) }
                .into_string()
                .unwrap(),
        );

        let exception = match self.0.new_object(
            &self
                .0
                .vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_DlErrorException)
                .into(),
            &stdlib_header::MethodId!(Exception::Constructor_String).into(),
            &[(&raw const message).cast_mut().cast()],
        ) {
            None => return false,
            Some(exception) => exception,
        };

        self.0.throw_exception(exception);

        true
    }
}

impl CPU {
    pub fn throw_helper<'a>(&'a self) -> &'a ThrowHelper {
        unsafe { &*(self as *const Self as *const ThrowHelper) }
    }
    pub fn throw_helper_mut<'a>(&'a mut self) -> &'a mut ThrowHelper {
        unsafe { &mut *(self as *mut Self as *mut ThrowHelper) }
    }
}

mod helpers {
    use crate::{
        type_system::class::Class, value::managed_reference::ManagedReference,
        virtual_machine::cpu::CPU,
    };

    impl CPU {
        pub fn has_exception(&self) -> bool {
            self.exception_manager().has_exception()
        }
        pub fn throw_exception(&mut self, exception: ManagedReference<Class>) {
            self.exception_manager_mut().set(exception);
        }
        pub fn get_exception(&self) -> ManagedReference<Class> {
            *self.exception_manager().get_exception()
        }
    }
}
