use std::{alloc::Layout, ptr::NonNull, sync::nonpoison::MappedRwLockReadGuard};

use enumflags2::make_bitflags;
use global::attrs::{CallConvention, MethodAttr, MethodImplementationFlags, Visibility};
use mem_leak_detector::LeakDetector;

use crate::{
    type_system::{
        assembly::{Assembly, TypeContainer},
        cached_type_reference::GenericCachedTypeReference,
        class::Class,
        generics::GenericCountRequirement,
        method::{ExceptionTable, Method, RuntimeInstruction},
        method_table::MethodTable,
    },
    virtual_machine::{VirtualMachine, global_vm},
};

pub macro g_core_type($i:ident) {
    $crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
        <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
            $crate::stdlib::CoreTypeId::$i,
        ),
    )
}

pub macro core_type_in($i:ident in $vm:ident) {
    $crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
        $vm.assembly_manager()
            .get_core_type($crate::stdlib::CoreTypeId::$i),
    )
}

pub macro g_core_class($i:ident) {
    <$crate::stdlib::CoreTypeId as $crate::stdlib::CoreTypeIdExt>::global_type_handle(
        $crate::stdlib::CoreTypeId::$i,
    )
    .unwrap_class()
}

pub macro core_class_in($i:ident in $vm:ident) {
    $vm.assembly_manager()
        .get_core_type($crate::stdlib::CoreTypeId::$i)
        .unwrap_class()
}

#[global_allocator]
pub static LEAK_DETECTOR: LeakDetector<std::alloc::System> = LeakDetector::system();

pub fn new_assembly_on<F: FnOnce(NonNull<Assembly>) -> Vec<TypeContainer>>(
    vm: &VirtualMachine,
    name: impl Into<widestring::Utf16String>,
    f: F,
) -> MappedRwLockReadGuard<'_, Assembly> {
    let id = vm
        .assembly_manager()
        .add_assembly(Assembly::new_for_adding(name.into(), false, f));
    vm.assembly_manager().get_assembly(id).unwrap()
}

pub fn new_global_assembly<F: FnOnce(NonNull<Assembly>) -> Vec<TypeContainer>>(
    name: impl Into<widestring::Utf16String>,
    f: F,
) -> MappedRwLockReadGuard<'static, Assembly> {
    new_assembly_on(global_vm(), name, f)
}

pub fn try_invoke_instructions_on(
    vm: &VirtualMachine,
    locals: Vec<GenericCachedTypeReference>,
    return_type: GenericCachedTypeReference,
    instructions: Vec<RuntimeInstruction>,
) -> (NonNull<u8>, Layout) {
    let assembly = new_assembly_on(vm, "Test::TryInvoke", |assembly| {
        vec![
            Class::new(
                assembly,
                widestring::utf16str!("Test::TryInvoke::Test").to_owned(),
                global::attr!(class Public {}),
                GenericCountRequirement::default(),
                Some(core_class_in!(System_Object in vm)),
                vec![],
                MethodTable::wrap_as_method_generator(|mt| {
                    vec![
                        Method::new(
                            mt,
                            widestring::utf16str!("__Test").to_owned(),
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

    let class = assembly.get_class(0).unwrap();
    let mt_ref = unsafe { class.as_ref().method_table_ref() };
    let method = mt_ref
        .find_first_method_by_name(widestring::utf16str!("__Test"))
        .unwrap();

    let mut cpu = vm.add_write_cpu();

    unsafe { method.as_ref().untyped_call(&mut cpu, None, &[]) }
}

pub fn try_invoke_instructions(
    locals: Vec<GenericCachedTypeReference>,
    return_type: GenericCachedTypeReference,
    instructions: Vec<RuntimeInstruction>,
) -> (NonNull<u8>, Layout) {
    try_invoke_instructions_on(global_vm(), locals, return_type, instructions)
}
