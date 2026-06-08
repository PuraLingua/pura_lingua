use std::{alloc::Layout, ptr::NonNull};

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt as _},
    type_system::{
        assembly::{Assembly, TypeContainer},
        class::Class,
        field::Field,
        generics::GenericCountRequirement,
        method::Method,
        method_table::MethodTable,
        type_handle::MaybeUnloadedTypeHandle,
    },
    virtual_machine::{EnsureGlobalVirtualMachineInitialized, global_vm},
};

#[test]
fn test_layout() {
    EnsureGlobalVirtualMachineInitialized();
    let vm = global_vm();
    let assembly_manager = vm.assembly_manager();
    assembly_manager.add_assembly(Assembly::new_for_adding(
        widestring::utf16str!("Test").to_owned(),
        false,
        |assem| {
            vec![TypeContainer::from(Class::new(
                assem,
                widestring::utf16str!("Test::Test").to_owned(),
                global::attr!(
                    class Public {}
                ),
                GenericCountRequirement::default(),
                None,
                Vec::new(),
                |class| {
                    MethodTable::new(class, |mt| {
                        vec![Method::default_sctor(
                            Some(mt),
                            global::attr!(method Public {Static}),
                        )]
                    })
                    .as_non_null_ptr()
                },
                vec![
                    Field::new(
                        widestring::utf16str!("a").to_owned(),
                        global::attr!(
                            field Public {}
                        ),
                        MaybeUnloadedTypeHandle::Unloaded(
                            CoreTypeId::System_UInt8.static_type_ref(),
                        ),
                    ),
                    Field::new(
                        widestring::utf16str!("b").to_owned(),
                        global::attr!(
                            field Public {}
                        ),
                        MaybeUnloadedTypeHandle::Unloaded(
                            CoreTypeId::System_UInt64.static_type_ref(),
                        ),
                    ),
                ],
                None,
                vec![],
                None,
            ))]
        },
    ));

    let assem = assembly_manager
        .get_assembly_by_name(widestring::utf16str!("Test"))
        .unwrap();

    let class = assem.get_type::<NonNull<Class>>(0).unwrap();

    let mt: &MethodTable<Class> = unsafe { class.as_ref().method_table_ref() };
    assert_eq!(
        mt.mem_layout(Default::default()),
        Layout::from_size_align(16, 8).unwrap()
    );
}
