use std::{alloc::Layout, ptr::NonNull, sync::MappedRwLockReadGuard};

use enumflags2::make_bitflags;
use global::{
    attrs::{CallConvention, MethodAttr, MethodImplementationFlags, Visibility},
    instruction::Instruction,
};
use mem_leak_detector::LeakDetector;

use crate::{
    type_system::{
        assembly::Assembly,
        class::Class,
        generics::GenericCountRequirement,
        method::{ExceptionTable, Method, MethodRef},
        method_table::MethodTable,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    virtual_machine::{CpuID, global_vm},
};

pub macro g_core_type($i:ident) {
    $crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
        <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
            $crate::stdlib::CoreTypeId::$i,
        ),
    )
}

pub macro g_core_class($i:ident) {
    <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
        $crate::stdlib::CoreTypeId::$i,
    )
    .unwrap_class()
}

#[global_allocator]
pub static LEAK_DETECTOR: LeakDetector<std::alloc::System> = LeakDetector::system();

pub fn new_global_assembly<F: FnOnce(NonNull<Assembly>) -> Vec<NonGenericTypeHandle>>(
    name: impl Into<String>,
    f: F,
) -> MappedRwLockReadGuard<'static, Assembly> {
    let id = global_vm()
        .assembly_manager()
        .add_assembly(Assembly::new_for_adding(name.into(), false, f));
    global_vm()
        .assembly_manager()
        .get_assembly(id)
        .unwrap()
        .unwrap()
}

pub fn try_invoke_instructions(
    locals: Vec<MaybeUnloadedTypeHandle>,
    return_type: MaybeUnloadedTypeHandle,
    instructions: Vec<Instruction<String, MaybeUnloadedTypeHandle, MethodRef, u32>>,
) -> (NonNull<u8>, Layout) {
    let assembly = new_global_assembly("Test::TryInvoke", |assembly| {
        vec![
            Class::new(
                assembly,
                "Test::TryInvoke::Test".to_owned(),
                global::attr!(class Public {}),
                GenericCountRequirement::default(),
                Some(g_core_class!(System_Object)),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![
                        Method::new(
                            mt,
                            "__Test".to_owned(),
                            MethodAttr::new(
                                Visibility::Public,
                                make_bitflags!(MethodImplementationFlags::{Static}),
                                None,
                                locals,
                            ),
                            GenericCountRequirement::default(),
                            vec![],
                            return_type,
                            CallConvention::PlatformDefault,
                            None,
                            instructions,
                            ExceptionTable::gen_new(),
                        ),
                        Method::default_sctor(Some(mt), global::attr!(method Public {Static})),
                    ]
                }),
                vec![],
                None,
                vec![],
                None,
            )
            .into(),
        ]
    });

    let class = assembly.get_class(0).unwrap().unwrap();
    let mt_ref = unsafe { class.as_ref().method_table_ref() };
    let method = mt_ref.find_first_method_by_name("__Test").unwrap();

    let mut cpu = CpuID::new_write_global();

    unsafe { method.as_ref().untyped_call(&mut cpu, None, &[]) }
}
