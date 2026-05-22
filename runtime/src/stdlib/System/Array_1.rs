use std::{ffi::c_void, ptr::NonNull};

use global::{
    attrs::CallConvention,
    instruction::{
        CommonWritePointer, IRegisterAddr, Instruction_Call, Instruction_Load, LoadContent,
    },
};

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{
        class::Class,
        generics::GenericCountRequirement,
        method::{ExceptionTable, Method},
        type_handle::{NonGenericTypeHandle, TypeHandle},
    },
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

pub extern "system" fn Destructor(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    super::Object::Destructor(cpu, method, this);
}

pub extern "system" fn ToString(
    cpu: &mut CPU,
    _method: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    let element_type = this
        .access::<ArrayAccessor>()
        .unwrap()
        .element_type_handle()
        .unwrap();
    let elements = unsafe {
        this.access_unchecked::<ArrayAccessor>()
            .as_raw_slices()
            .unwrap()
    };
    let mut strings = Vec::with_capacity(elements.len());
    match element_type {
        NonGenericTypeHandle::Class(_) | NonGenericTypeHandle::Interface(_) => {
            for ele_b in elements {
                let ele = unsafe {
                    ele_b
                        .as_ptr()
                        .cast::<ManagedReference<Class>>()
                        .as_ref_unchecked()
                };

                assert!(!ele.is_null());

                let mt = unsafe { ele.method_table_ref_unchecked() };
                let method = mt
                    .get_method(stdlib_header::System::Object::MethodId::ToString as _)
                    .unwrap();

                let mut res = unsafe {
                    method.as_ref().typed_res_call::<ManagedReference<Class>>(
                        cpu,
                        Some(NonNull::from_ref(ele).cast()),
                        &[],
                    )
                };

                strings.push(
                    res.access::<StringAccessor>()
                        .unwrap()
                        .to_string_lossy()
                        .unwrap(),
                );

                res.destroy(cpu);
            }
        }
        NonGenericTypeHandle::Struct(ty) => {
            for ele in elements {
                let ele_p = ele.as_ptr();
                let mt: &crate::type_system::method_table::MethodTable<
                    crate::type_system::r#struct::Struct,
                > = unsafe { ty.as_ref().method_table_ref() };
                let method = mt
                    .find_first_method_by_name(widestring::utf16str!("ToString()"))
                    .unwrap();

                let mut res = unsafe {
                    method.as_ref().typed_res_call::<ManagedReference<Class>>(
                        cpu,
                        None,
                        &[(&raw const ele_p).cast_mut().cast()],
                    )
                };

                strings.push(
                    res.access::<StringAccessor>()
                        .unwrap()
                        .to_string_lossy()
                        .unwrap(),
                );

                res.destroy(cpu);
            }
        }
    }

    let s = format!("[{}]", strings.join(", "));
    ManagedReference::new_string(cpu, &s)
}

pub extern "system" fn GetReference(
    _: &CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
    index: usize,
) -> Option<NonNull<u8>> {
    this.access::<ArrayAccessor>()
        .and_then(ArrayAccessor::as_raw_slices)
        .and_then(|mut x| x.nth(index))
        .map(|x| unsafe { NonNull::new_unchecked(x.as_ptr().cast_mut()) })
}

pub extern "system" fn get_Index(
    cpu: &mut CPU,
    _: &Method<Class>,
    this: &ManagedReference<Class>,
    index: usize,
    return_buffer: NonNull<c_void>,
) {
    let this_arr = this.access::<ArrayAccessor>().unwrap();
    let Some(length) = this_arr.len() else {
        debug_assert!(cpu.throw_helper_mut().null_reference());
        return;
    };
    let element = match this_arr.as_raw_slices().unwrap().nth(index) {
        Some(element) => element,
        None => {
            debug_assert!(cpu.throw_helper_mut().index_out_of_range(index, length));
            return;
        }
    };
    unsafe {
        return_buffer
            .cast::<u8>()
            .copy_from(NonNull::from_ref(element).as_non_null_ptr(), length);
    }
}

macro define_registers($($name:ident)*) {$(
    const $name: global::instruction::ShortRegisterAddr = global::instruction::ShortRegisterAddr::new(${index()});
)*}

_define_class!(
    fn load(assembly, mt, method_info)
    Array_1
#methods(TMethodId):
    Destructor => common_new_method!(mt TMethodId Destructor Destructor);
    ToString => common_new_method!(mt TMethodId ToString ToString);
    GetReference => common_new_method!(mt TMethodId GetReference GetReference);
    get_Index => common_new_method!(mt TMethodId get_Index get_Index);
    set_Index => {
        define_registers!(
            this_addr
            arg_Index
            arg_Value
            t_size
            pointer2target
        );
        Method::new(
            mt,
            widestring::Utf16String::from_str(TMethodId::set_Index.get_name()),
            super::map_method_attr(TMethodId::set_Index.get_attr()),
            GenericCountRequirement::default(),
            TMethodId::set_Index.get_parameters()
                .into_iter()
                .map(super::map_parameter)
                .collect(),
            TMethodId::set_Index.get_return_type().into(),
            CallConvention::PlatformDefault,
            None,
            {
                use global::instruction::Instruction;
                vec![
                    Instruction::SLoad(Instruction_Load { addr: this_addr, content: LoadContent::This }),
                    Instruction::SLoad(Instruction_Load { addr: arg_Index, content: LoadContent::Arg(0) }),
                    Instruction::SLoad(Instruction_Load { addr: arg_Value, content: LoadContent::Arg(1) }),

                    Instruction::SLoad(Instruction_Load {
                        addr: t_size,
                        content: LoadContent::TypeValueSize(TypeHandle::TypeGeneric(0).into()),
                    }),

                    Instruction::SCall(Instruction_Call::InstanceCall {
                        val: this_addr,
                        method: TMethodId::GetReference.into(),
                        args: vec![arg_Index],
                        ret_at: pointer2target,
                    }),
                    Instruction::SWritePointer(CommonWritePointer {
                        source: arg_Value,
                        size: t_size,
                        ptr: pointer2target,
                    }),
                ]
            },
            ExceptionTable::gen_new(),
        )
    };
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
