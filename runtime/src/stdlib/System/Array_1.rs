use std::ptr::NonNull;

use global::{
    attrs::CallConvention,
    dt_println,
    instruction::{
        CommonReadPointerTo, CommonWritePointer, IRegisterAddr, Instruction_Call, Instruction_Load,
        LoadContent,
    },
};

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{
        class::Class,
        method::Method,
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
    dt_println!("Dropping Array`1");

    super::Object::Destructor(cpu, method, this);
}

pub extern "system" fn ToString(
    cpu: &mut CPU,
    method: &Method<Class>,
    this: &ManagedReference<Class>,
) -> ManagedReference<Class> {
    let element_type = this
        .access::<ArrayAccessor>()
        .unwrap()
        .element_type_handle()
        .unwrap();
    let element_type = element_type.get_non_generic_with_method(method);
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
                let mt = unsafe { ty.as_ref().method_table_ref() };
                let method = mt.find_first_method_by_name("ToString()").unwrap();

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

pub extern "system" fn GetPointerOfIndex(
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

macro define_registers($($name:ident)*) {$(
    const $name: global::instruction::ShortRegisterAddr = global::instruction::ShortRegisterAddr::new(${index()});
)*}

_define_class!(
    fn load(assembly, mt, method_info)
    Array_1
#methods(TMethodId):
    Destructor => common_new_method!(mt TMethodId Destructor Destructor);
    ToString => common_new_method!(mt TMethodId ToString ToString);
    GetPointerOfIndex => common_new_method!(mt TMethodId GetPointerOfIndex GetPointerOfIndex);
    get_Index => {
        define_registers!(
            this_addr
            arg_Index
            ptr2result
            t_size
            result
        );
        Box::new(
            Method::new(
                mt,
                TMethodId::get_Index.get_name().to_owned(),
                super::map_method_attr(TMethodId::get_Index.get_attr()),
                TMethodId::get_Index.get_parameters()
                    .into_iter()
                    .map(super::map_parameter)
                    .collect(),
                TMethodId::get_Index.get_return_type().into(),
                CallConvention::PlatformDefault,
                None,
                {
                    use global::instruction::Instruction;
                    vec![
                        Instruction::SLoad(Instruction_Load {
                            addr: this_addr,
                            content: LoadContent::This,
                        }),
                        Instruction::SLoad(Instruction_Load {
                            addr: arg_Index,
                            content: LoadContent::Arg(0),
                        }),
                        Instruction::SCall(global::instruction::Instruction_Call::InstanceCall {
                            val: this_addr,
                            method: TMethodId::GetPointerOfIndex.into(),
                            args: vec![arg_Index],
                            ret_at: ptr2result,
                        }),
                        Instruction::SLoad(Instruction_Load {
                            addr: t_size,
                            content: LoadContent::TypeValueSize(TypeHandle::Generic(0).into()),
                        }),
                        Instruction::SReadPointerTo(CommonReadPointerTo {
                            ptr: ptr2result,
                            size: t_size,
                            destination: result,
                        }),
                        Instruction::SReturnVal {
                            register_addr: result,
                        },
                    ]
                },
            )
        )
    };
    set_Index => {
        define_registers!(
            this_addr
            arg_Index
            arg_Value
            t_size
            pointer2target
        );
        Box::new(
            Method::new(
                mt,
                TMethodId::set_Index.get_name().to_owned(),
                super::map_method_attr(TMethodId::set_Index.get_attr()),
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
                            content: LoadContent::TypeValueSize(TypeHandle::Generic(0).into()),
                        }),

                        Instruction::SCall(Instruction_Call::InstanceCall {
                            val: this_addr,
                            method: TMethodId::GetPointerOfIndex.into(),
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
            )
        )
    };
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
);
