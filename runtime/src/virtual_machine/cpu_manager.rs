use std::{
    cell::SyncUnsafeCell,
    mem::{MaybeUninit, offset_of},
    num::NonZero,
    pin::Pin,
    ptr::NonNull,
    sync::nonpoison::{Mutex, RwLock, RwLockWriteGuard},
};

use crate::virtual_machine::{VirtualMachine, cpu::CPU};

pub struct CPUManager {
    vm: NonNull<VirtualMachine>,

    cpu_lock: Mutex<()>,
    #[allow(clippy::vec_box)]
    central_processing_units: SyncUnsafeCell<Vec<Pin<Box<RwLock<CPU>>>>>,
    cpu_for_static: Pin<Box<RwLock<CPU>>>,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CpuID {
    StaticCPU = 0,
    Common(NonZero<usize>) = 1,
}

impl CpuID {
    pub fn new_global() -> Pin<&'static RwLock<CPU>> {
        super::global_vm().add_cpu().as_global_cpu().unwrap()
    }
    pub fn new_write_global() -> RwLockWriteGuard<'static, CPU> {
        super::global_vm().add_cpu().as_global_write_cpu().unwrap()
    }
    pub fn as_global_cpu(&self) -> Option<Pin<&'static RwLock<CPU>>> {
        super::global_vm().get_cpu(*self)
    }
    pub fn as_global_write_cpu(&self) -> Option<RwLockWriteGuard<'static, CPU>> {
        self.as_global_cpu().map(Pin::get_ref).map(|x| x.write())
    }
}

impl CPUManager {
    pub fn construct_in(vm: NonNull<VirtualMachine>) {
        unsafe {
            vm.byte_add(offset_of!(VirtualMachine, cpu_manager))
                .cast::<CPUManager>()
                .write(Self {
                    vm,

                    cpu_lock: Mutex::new(()),
                    central_processing_units: SyncUnsafeCell::new(Vec::new()),
                    #[expect(invalid_value, reason = "It will be init then")]
                    cpu_for_static: MaybeUninit::uninit().assume_init(),
                });
        }

        let this: &mut Self = unsafe { NonNull::from_ref(vm.as_ref().cpu_manager()).as_mut() };
        unsafe {
            NonNull::from_mut(this)
                .byte_add(offset_of!(Self, cpu_for_static))
                .cast()
                .write(CPU::new(NonNull::from_mut(this)));
        }
    }
}

impl CPUManager {
    pub fn cpu_for_static(&self) -> Pin<&RwLock<CPU>> {
        Pin::as_ref(&self.cpu_for_static)
    }

    pub fn write_cpu_for_static<'a>(&'a self) -> RwLockWriteGuard<'a, CPU> {
        self.cpu_for_static.write()
    }

    #[must_use]
    pub fn add_cpu(&self) -> CpuID {
        let _guard = self.cpu_lock.lock();
        let central_processing_units =
            unsafe { self.central_processing_units.get().as_mut_unchecked() };
        let index = central_processing_units.len();
        central_processing_units.push(CPU::new(NonNull::from_ref(self)));
        CpuID::Common(unsafe { NonZero::new_unchecked(index + 1) })
    }

    pub fn get_cpu(&self, index: CpuID) -> Option<Pin<&RwLock<CPU>>> {
        match index {
            CpuID::StaticCPU => Some(self.cpu_for_static()),
            CpuID::Common(index) => {
                let _guard = self.cpu_lock.lock();
                let central_processing_units =
                    unsafe { self.central_processing_units.get().as_ref_unchecked() };
                central_processing_units
                    .get(index.get() - 1)
                    .map(|x| Pin::as_ref(x))
            }
        }
    }
}

impl CPUManager {
    pub fn vm_ref(&self) -> &VirtualMachine {
        unsafe { self.vm.as_ref() }
    }
}
