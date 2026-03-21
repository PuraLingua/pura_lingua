use std::{
    alloc::{Allocator, Layout},
    pin::Pin,
    ptr::NonNull,
    sync::{MappedRwLockReadGuard, RwLock},
};

use pura_lingua::runtime::{
    self,
    type_system::assembly_manager::AssemblyManager,
    virtual_machine::{CpuID, VirtualMachine, cpu::CPU},
};

#[unsafe(no_mangle)]
pub extern "C" fn EnsureGlobalVirtualMachineInitialized() {
    runtime::virtual_machine::EnsureGlobalVirtualMachineInitialized();
}

#[unsafe(no_mangle)]
pub extern "C" fn IsGlobalVirtualMachineInitialized() -> bool {
    runtime::virtual_machine::is_global_vm_init()
}

// Constructors

/// Returns borrowed pointer
#[unsafe(no_mangle)]
pub extern "C" fn GlobalVirtualMachine() -> NonNull<VirtualMachine> {
    NonNull::from_mut(runtime::virtual_machine::global_vm())
}

/// Returns borrowed pointer
#[unsafe(no_mangle)]
pub extern "C" fn GlobalVirtualMachineUnchecked() -> NonNull<VirtualMachine> {
    unsafe { NonNull::from_mut(runtime::virtual_machine::global_vm_unchecked()) }
}

/// Returns owned pointer
#[unsafe(no_mangle)]
pub extern "C" fn NewVirtualMachine() -> NonNull<VirtualMachine> {
    VirtualMachine::new_system().as_non_null_ptr()
}

// Methods

#[unsafe(no_mangle)]
pub extern "C" fn VirtualMachine_AddCPU(vm: &VirtualMachine) -> CpuID {
    vm.add_cpu()
}

#[unsafe(no_mangle)]
pub extern "C" fn VirtualMachine_GetCPULock<'a>(
    vm: &'a VirtualMachine,
    index: CpuID,
) -> Option<Pin<&'a RwLock<CPU>>> {
    vm.get_cpu(index)
}

/// Static method
#[unsafe(no_mangle)]
pub extern "C" fn VirtualMachine_DropCPUGuard<'a>(guard: NonNull<MappedRwLockReadGuard<'a, CPU>>) {
    unsafe {
        drop(Box::from_non_null(guard));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn VirtualMachine_GetAssemblyManagerRef(vm: &VirtualMachine) -> &AssemblyManager {
    vm.assembly_manager()
}

pub mod cpu;

// Destructors

#[unsafe(no_mangle)]
pub extern "C" fn VirtualMachine_Drop(vm: *mut VirtualMachine) {
    unsafe {
        vm.drop_in_place();
        std::alloc::System.deallocate(
            NonNull::new_unchecked(vm).cast(),
            Layout::new::<VirtualMachine>(),
        );
    }
}
