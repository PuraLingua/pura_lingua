use std::{alloc::Layout, ptr::NonNull};

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt as _},
    test_utils,
    type_system::{
        assembly::TypeContainer, class::Class, field::Field, generics::GenericCountRequirement,
        method::Method, method_table::MethodTable, type_handle::MaybeUnloadedTypeHandle,
    },
    virtual_machine::create_vm_on_stack,
};

#[test]
fn test_layout() {
    create_vm_on_stack!(vm);

    let assem = test_utils::new_assembly_on(&vm, "Test", |assem| {
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
                    MaybeUnloadedTypeHandle::Unloaded(CoreTypeId::System_UInt8.static_type_ref()),
                ),
                Field::new(
                    widestring::utf16str!("b").to_owned(),
                    global::attr!(
                        field Public {}
                    ),
                    MaybeUnloadedTypeHandle::Unloaded(CoreTypeId::System_UInt64.static_type_ref()),
                ),
            ],
            None,
            vec![],
            None,
        ))]
    });

    let class = assem.get_type::<NonNull<Class>>(0).unwrap();

    let mt: &MethodTable<Class> = unsafe { class.as_ref().method_table_ref() };
    assert_eq!(
        mt.mem_layout(Default::default()),
        Layout::from_size_align(16, 8).unwrap()
    );
}
