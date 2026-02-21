use crate::type_system::{class::Class, r#struct::Struct, type_handle::NonGenericTypeHandleKind};

use super::CPU;

impl CPU {
    pub fn gc_collect(&self) {
        let mut mem_records = self.write_mem_records().unwrap();
        let mut call_stack = self.write_call_stack().unwrap();
        call_stack.mark_all();
        drop(call_stack);

        for mem_record in mem_records.iter_mut() {
            match mem_record.kind {
                NonGenericTypeHandleKind::Class => {
                    let mut ptr = mem_record.ptr.cast::<Class>();
                    if ptr.header().is_some_and(|x| !x.is_marked()) {
                        ptr.destroy(self);
                        mem_record.to_be_dropped = true;
                    }
                }
                NonGenericTypeHandleKind::Struct => {
                    let mut ptr = mem_record.ptr.cast::<Struct>();
                    if ptr.header().is_some_and(|x| !x.is_marked()) {
                        ptr.destroy(self);
                        mem_record.to_be_dropped = true;
                    }
                }
            }
        }
        mem_records.retain(|x: &super::MemoryRecord| !x.to_be_dropped);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use crate::{
        test_utils::g_core_class,
        value::managed_reference::{ArrayAccessor, ManagedReference},
        virtual_machine::{EnsureVirtualMachineInitialized, global_vm},
    };

    use super::*;

    #[test]
    fn gc() {
        EnsureVirtualMachineInitialized();

        let vm = global_vm();
        let cpu_id = vm.add_cpu();
        let cpu = cpu_id.as_global_cpu().unwrap();
        let string_t = g_core_class!(System_String);
        let string_mt = unsafe { string_t.as_ref().method_table_ref() };

        let mut array_obj = ManagedReference::alloc_array(&cpu, NonNull::from_ref(string_mt), 10);
        unsafe {
            for (ele_i, ele) in array_obj
                .access_unchecked_mut::<ArrayAccessor>()
                .as_slice_mut::<ManagedReference<Class>>()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                *ele = ManagedReference::new_string(&cpu, &format!("VARIABLE:{ele_i}"));
            }
        }
        cpu.gc_collect();
    }
}
