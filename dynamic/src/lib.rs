#![feature(once_cell_try)]
#![feature(deref_pure_trait)]

use std::{
    ops::{Deref, DerefMut, DerefPure},
    sync::OnceLock,
};

pub struct SingletonDynamic<T> {
    pub data: T,
}

impl<T> SingletonDynamic<T> {
    pub fn new(
        filename: impl libloading::AsFilename,
        ctor_name: impl libloading::AsSymbolName,
    ) -> Result<Self, libloading::Error> {
        static SINGLETON: OnceLock<libloading::Library> = OnceLock::new();
        let lib = SINGLETON.get_or_try_init(|| unsafe { libloading::Library::new(filename) })?;
        let ctor = unsafe { lib.get::<extern "C" fn() -> T>(ctor_name)? };
        let data = ctor();
        Ok(Self { data })
    }
}

impl<T> Deref for SingletonDynamic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for SingletonDynamic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

unsafe impl<T> DerefPure for SingletonDynamic<T> {}
