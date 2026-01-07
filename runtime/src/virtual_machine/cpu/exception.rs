use crate::{type_system::class::Class, value::managed_reference::ManagedReference};

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
