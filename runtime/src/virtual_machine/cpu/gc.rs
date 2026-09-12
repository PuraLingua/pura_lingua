use std::ptr::NonNull;

use crate::type_system::{class::Class, r#struct::Struct, type_handle::NonGenericTypeHandleKind};

use super::CPU;

impl CPU {
    #[allow(unreachable_code)]
    pub fn gc_collect(&mut self) {
        eprintln!("GC is incomplete");
        return;

        self.call_stack.set_marker(true);

        // Little hack
        for mem_record in self.mem_records.iter() {
            match mem_record.kind {
                NonGenericTypeHandleKind::Class | NonGenericTypeHandleKind::Interface => {
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

        self.call_stack.set_marker(false);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use crate::{
        test_utils::core_class_in,
        value::managed_reference::{ArrayAccessor, ManagedReference},
        virtual_machine::create_vm_on_stack,
    };

    use super::*;

    #[test]
    fn gc() {
        create_vm_on_stack!(vm);

        let mut cpu = vm.add_write_cpu();
        let string_t = core_class_in!(System_String in vm);
        let string_mt = unsafe { string_t.as_ref().method_table_ref() };

        let mut array_obj =
            ManagedReference::alloc_array(&mut cpu, NonNull::from_ref(string_mt), 10);
        unsafe {
            for (ele_i, ele) in array_obj
                .access_unchecked_mut::<ArrayAccessor>()
                .as_slice_mut::<ManagedReference<Class>>()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                *ele = ManagedReference::new_string(&mut cpu, &format!("VARIABLE:{ele_i}"));
            }
        }
        cpu.gc_collect();
    }
}
