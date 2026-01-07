use std::ptr::NonNull;

use crate::{
    stdlib::System_Object_MethodId,
    type_system::{class::Class, method::Method, type_handle::NonGenericTypeHandle},
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

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
            .to_raw_slices()
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
    ManagedReference::new_string(cpu, s)
}

#[cfg(test)]
mod tests {
    use crate::{
        stdlib::CoreTypeId,
        virtual_machine::{EnsureVirtualMachineInitialized, global_vm},
    };

    use super::*;

    #[test]
    fn test_to_string() {
        EnsureVirtualMachineInitialized();

        let cpu_id = global_vm().add_cpu();
        let cpu = cpu_id.as_global_cpu().unwrap();
        let string_t = CoreTypeId::System_String
            .global_type_handle()
            .unwrap_class();
        let s1 = ManagedReference::new_string(&cpu, "aaa".to_owned());
        let s2 = ManagedReference::new_string(&cpu, "bbb".to_owned());
        let mut arr =
            ManagedReference::alloc_array(&cpu, unsafe { *string_t.as_ref().method_table() }, 2);

        unsafe {
            println!("Arr MT: {}", arr.method_table_ref_unchecked().display());
            let array_accessor = arr.access_unchecked_mut::<ArrayAccessor>();
            let slice = array_accessor
                .as_slice_mut::<ManagedReference<Class>>()
                .unwrap();
            slice[0] = s1;
            slice[1] = s2;
        }

        let ToString_m = unsafe {
            arr.method_table_ref_unchecked()
                .get_method(System_Object_MethodId::ToString as _)
                .unwrap()
        };

        let s = unsafe {
            let arr_r = &arr;
            ToString_m
                .as_ref()
                .typed_res_call::<ManagedReference<Class>>(
                    &cpu,
                    Some(NonNull::from_ref(arr_r).cast()),
                    &[],
                )
        };

        dbg!(s.access::<StringAccessor>().unwrap().to_string_lossy());
    }
}
