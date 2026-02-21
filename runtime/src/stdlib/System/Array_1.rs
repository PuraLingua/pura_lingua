use std::ptr::NonNull;

use global::dt_println;

use crate::{
    stdlib::System_Object_MethodId,
    type_system::{class::Class, method::Method, type_handle::NonGenericTypeHandle},
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

#[cfg(test)]
mod tests;

pub extern "system" fn Destructor(
    cpu: &CPU,
    method: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    dt_println!("Dropping Array`1");

    super::Object::Destructor(cpu, method, this);
}

pub extern "system" fn ToString(
    cpu: &CPU,
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
        NonGenericTypeHandle::Class(_) => {
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
                    .get_method(System_Object_MethodId::ToString as _)
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
