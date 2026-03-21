use std::ptr::NonNull;

use pura_lingua::runtime::{
    type_system::{assembly::Assembly, assembly_manager::AssemblyManager},
    virtual_machine::VirtualMachine,
};

#[unsafe(no_mangle)]
pub extern "C" fn AssemblyManager_VirtualMachineRef(this: &AssemblyManager) -> &VirtualMachine {
    this.vm_ref()
}

#[unsafe(no_mangle)]
pub extern "C" fn AssemblyManager_AddAssembly(
    this: &AssemblyManager,
    // Owned
    assembly: NonNull<Assembly>,
) -> usize {
    unsafe { this.add_assembly(Box::from_non_null(assembly)) }
}
