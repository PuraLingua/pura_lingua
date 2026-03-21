use std::ptr::NonNull;

use crate::type_system::{class::Class, r#struct::Struct, type_handle::NonGenericTypeHandleKind};

use super::CPU;

impl CPU {
    pub fn gc_collect(&mut self) {
        self.call_stack.mark_all();

        // Little hack
        for mem_record in self.mem_records.iter() {
            match mem_record.kind {
                NonGenericTypeHandleKind::Class => {
                    let mut ptr = mem_record.ptr.cast::<Class>();
                    if ptr.header().is_some_and(|x| !x.is_marked()) {
                        ptr.destroy(unsafe { NonNull::from_ref(self).as_mut() });
                        unsafe {
                            NonNull::from_ref(&mem_record.to_be_dropped).write(true);
                        }
                    }
                }
                NonGenericTypeHandleKind::Struct => {
                    let mut ptr = mem_record.ptr.cast::<Struct>();
                    if ptr.header().is_some_and(|x| !x.is_marked()) {
                        ptr.destroy(unsafe { NonNull::from_ref(self).as_mut() });
                        unsafe {
                            NonNull::from_ref(&mem_record.to_be_dropped).write(true);
                        }
                    }
                }
            }
        }
        self.mem_records
            .retain(|x: &super::MemoryRecord| !x.to_be_dropped);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use crate::{
        test_utils::g_core_class,
        value::managed_reference::{ArrayAccessor, ManagedReference},
        virtual_machine::{EnsureGlobalVirtualMachineInitialized, global_vm},
    };

    use super::*;

    #[test]
    fn gc() {
        EnsureGlobalVirtualMachineInitialized();

        let vm = global_vm();
        let cpu_id = vm.add_cpu();
        let cpu = cpu_id.as_global_cpu().unwrap();
        let mut cpu_write = cpu.write().unwrap();
        let string_t = g_core_class!(System_String);
        let string_mt = unsafe { string_t.as_ref().method_table_ref() };

        let mut array_obj =
            ManagedReference::alloc_array(&mut cpu_write, NonNull::from_ref(string_mt), 10);
        unsafe {
            for (ele_i, ele) in array_obj
                .access_unchecked_mut::<ArrayAccessor>()
                .as_slice_mut::<ManagedReference<Class>>()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                *ele = ManagedReference::new_string(&mut cpu_write, &format!("VARIABLE:{ele_i}"));
            }
        }
        cpu_write.gc_collect();
    }
}
