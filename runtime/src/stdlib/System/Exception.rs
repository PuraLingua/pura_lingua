use enumflags2::make_bitflags;
use stdlib_header::System::Exception::FieldId;

use crate::{
    stdlib::{
        CoreTypeId,
        System::{_define_class, common_new_method, default_sctor},
    },
    type_system::{
        class::Class,
        method::{Method, MethodDisplayOptions},
    },
    value::managed_reference::{ArrayAccessor, FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Constructor_String(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    message: ManagedReference<Class>,
) {
    let string_t = cpu
        .vm_ref()
        .assembly_manager()
        .get_core_type(CoreTypeId::System_String)
        .unwrap_class();

    let f_message_mut = this
        .const_access_mut::<FieldAccessor<_>>()
        .typed_field_mut::<ManagedReference<Class>>(FieldId::Message as _, Default::default())
        .unwrap();

    *f_message_mut = message;

    let f_stack_trace_mut = this
        .const_access_mut::<FieldAccessor<_>>()
        .typed_field_mut::<ManagedReference<Class>>(FieldId::StackTrace as _, Default::default())
        .unwrap();

    let stack_trace_rs = cpu.capture_with_options(
        make_bitflags!(MethodDisplayOptions::{WithCallConvention | WithReturn | WithArgs}),
    );
    *f_stack_trace_mut = ManagedReference::alloc_array(
        cpu,
        unsafe { *string_t.as_ref().method_table() },
        stack_trace_rs.len(),
    );

    // Safety: It's allocated with `alloc_array`
    let f_stack_trace_slice_mut = unsafe {
        f_stack_trace_mut
            .access_unchecked_mut::<ArrayAccessor>()
            .as_slice_mut::<ManagedReference<Class>>()
            .unwrap()
    };
    for (ind, stack_name) in stack_trace_rs.into_iter().enumerate() {
        f_stack_trace_slice_mut[ind] = ManagedReference::new_string(cpu, &stack_name);
    }
}

pub extern "system" fn ToString(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    super::Object::ToString(cpu, method, this)
}

_define_class!(
    fn load(assembly, mt, method_info)
    Exception
#methods(TMethodId):
    ToString => common_new_method!(mt TMethodId ToString ToString);
    Constructor_String => common_new_method!(mt TMethodId Constructor_String Constructor_String);
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use stdlib_header::System::Exception::MethodId;

    use crate::{
        stdlib::{CoreTypeId, CoreTypeIdExt as _},
        value::managed_reference::{ArrayAccessor, StringAccessor},
        virtual_machine::{CpuID, EnsureGlobalVirtualMachineInitialized, global_vm},
    };

    use super::*;

    #[test]
    fn test_construct_exception() {
        EnsureGlobalVirtualMachineInitialized();

        let vm = global_vm();
        let u8_t = CoreTypeId::System_UInt8
            .global_type_handle()
            .unwrap_struct();
        let u16_t = CoreTypeId::System_UInt16
            .global_type_handle()
            .unwrap_struct();
        let array_t = CoreTypeId::System_Array_1
            .global_type_handle()
            .unwrap_class();
        let object_t = CoreTypeId::System_Object
            .global_type_handle()
            .unwrap_class();
        let mut cpu = CpuID::new_write_global();
        unsafe {
            cpu.push_call_stack_native(
                u8_t.as_ref()
                    .method_table_ref()
                    .get_method(stdlib_header::System::UInt8::StaticMethodId::ToString as _)
                    .unwrap()
                    .as_ref(),
            );

            cpu.push_call_stack_native(
                u16_t
                    .as_ref()
                    .method_table_ref()
                    .get_method(stdlib_header::System::UInt16::StaticMethodId::ToString as _)
                    .unwrap()
                    .as_ref(),
            );
            cpu.push_call_stack_native(
                array_t
                    .as_ref()
                    .method_table_ref()
                    .get_method(stdlib_header::System::Array_1::MethodId::ToString as _)
                    .unwrap()
                    .as_ref(),
            );
            cpu.push_call_stack_native(
                object_t
                    .as_ref()
                    .method_table_ref()
                    .get_method(stdlib_header::System::Object::MethodId::ToString as _)
                    .unwrap()
                    .as_ref(),
            );
        }

        let exception_t = vm
            .assembly_manager()
            .get_core_type(CoreTypeId::System_Exception)
            .unwrap_class();

        let exception_mt = unsafe { exception_t.as_ref().method_table_ref() };

        let exception_ptr = ManagedReference::<Class>::common_alloc(
            &mut cpu,
            NonNull::from_ref(exception_mt),
            false,
        );

        let method = exception_mt
            .get_method(MethodId::Constructor_String as _)
            .unwrap();

        let message = ManagedReference::new_string(&mut cpu, "AAA");
        unsafe {
            method.as_ref().typed_res_call::<()>(
                &mut cpu,
                Some(NonNull::from_ref(&exception_ptr).cast()),
                &[(&raw const message).cast_mut().cast()],
            );
        }
        let f_message = exception_ptr
            .const_access::<FieldAccessor<_>>()
            .typed_field::<ManagedReference<Class>>(FieldId::Message as _, Default::default())
            .unwrap();

        assert_eq!(
            f_message
                .access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap(),
            widestring::u16cstr!("AAA"),
        );

        let f_stack_trace = exception_ptr
            .const_access::<FieldAccessor<_>>()
            .typed_field::<ManagedReference<Class>>(FieldId::StackTrace as _, Default::default())
            .unwrap();
        unsafe {
            for stack in f_stack_trace
                .access::<ArrayAccessor>()
                .unwrap()
                .as_slice::<ManagedReference<Class>>()
                .unwrap()
            {
                println!(
                    "{}",
                    stack
                        .access::<StringAccessor>()
                        .unwrap()
                        .get_str()
                        .unwrap()
                        .display()
                );
            }
        }
    }
}
