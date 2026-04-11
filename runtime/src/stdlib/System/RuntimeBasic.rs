use std::alloc::Layout;

use line_ending::LineEnding;
use stdlib_header::System::RuntimeBasic::FieldId;

use crate::{
    stdlib::System::{_define_class, common_new_method},
    type_system::{class::Class, method::Method},
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::CPU,
};

pub extern "system" fn StaticConstructor(cpu: &mut CPU, method: &Method<Class>) {
    let (new_line_out, new_line_layout) = cpu
        .vm_ref()
        .get_static_field(
            method.require_method_table_ref().ty.into(),
            FieldId::NewLine as _,
        )
        .unwrap();

    debug_assert_eq!(new_line_layout, Layout::new::<ManagedReference<Class>>());
    unsafe {
        new_line_out
            .cast::<ManagedReference<Class>>()
            .write(ManagedReference::new_string(
                cpu,
                LineEnding::from_current_platform().as_str(),
            ))
    }
}

mod allocation;

_define_class!(
    fn load(assembly, mt, method_info)
    RuntimeBasic
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => Box::new(Method::create_sctor(
        Some(mt),
        super::map_method_attr(TStaticMethodId::StaticConstructor.get_attr()),
        StaticConstructor,
    ));

    /* #region Allocation */
    Global_Allocate => common_new_method!(mt TStaticMethodId Global_Allocate allocation::Global_Allocate);
    Global_AllocateZeroed => common_new_method!(mt TStaticMethodId Global_AllocateZeroed allocation::Global_AllocateZeroed);
    Global_Deallocate => common_new_method!(mt TStaticMethodId Global_Deallocate allocation::Global_Deallocate);
    Global_Grow => common_new_method!(mt TStaticMethodId Global_Grow allocation::Global_Grow);
    Global_GrowZeroed => common_new_method!(mt TStaticMethodId Global_GrowZeroed allocation::Global_GrowZeroed);
    Global_Shrink => common_new_method!(mt TStaticMethodId Global_Shrink allocation::Global_Shrink);
    /* #endregion */
);
