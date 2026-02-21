use std::ptr::NonNull;

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt as _},
    type_system::{
        assembly::Assembly,
        class::Class,
        field::Field,
        method::Method,
        method_table::MethodTable,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    virtual_machine::{EnsureVirtualMachineInitialized, global_vm},
};

#[test]
fn test_layout() {
    EnsureVirtualMachineInitialized();
    let vm = global_vm();
    let assembly_manager = vm.assembly_manager();
    assembly_manager.add_assembly(Assembly::new_for_adding(
        "Test".to_owned(),
        false,
        |assem| {
            vec![NonGenericTypeHandle::Class(
                Class::new(
                    assem,
                    "Test::Test".to_owned(),
                    global::attr!(
                        class Public {}
                    ),
                    None,
                    |class| {
                        MethodTable::new(class, |mt| {
                            vec![Box::new(Method::default_sctor(
                                Some(mt),
                                global::attr!(method Public {Static}),
                            ))]
                        })
                        .as_non_null_ptr()
                    },
                    vec![
                        Field::new(
                            "a".to_owned(),
                            global::attr!(
                                field Public {}
                            ),
                            MaybeUnloadedTypeHandle::Unloaded(
                                CoreTypeId::System_UInt8.static_type_ref(),
                            ),
                        ),
                        Field::new(
                            "b".to_owned(),
                            global::attr!(
                                field Public {}
                            ),
                            MaybeUnloadedTypeHandle::Unloaded(
                                CoreTypeId::System_UInt64.static_type_ref(),
                            ),
                        ),
                    ],
                    None,
                    None,
                )
                .as_non_null_ptr(),
            )]
        },
    ));

    let assem = assembly_manager
        .get_assembly_by_name("Test")
        .unwrap()
        .unwrap();

    let class = assem.get_type::<NonNull<Class>>(0).unwrap().unwrap();

    let mt = unsafe { class.as_ref().method_table_ref() };
    dbg!(mt.mem_layout(Default::default()));
}
