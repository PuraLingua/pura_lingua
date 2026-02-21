use std::{
    alloc::{Allocator, Global, Layout},
    collections::HashMap,
    mem::{MaybeUninit, offset_of},
    num::NonZero,
    ptr::{NonNull, Unique},
    sync::{MappedRwLockReadGuard, Once, RwLock, RwLockReadGuard},
};

use cpu::CPU;
use global::{ThreadSafe, getset::Getters};

use crate::{
    memory::alloc_type,
    type_system::{
        assembly_manager::AssemblyManager, class::Class, get_traits::GetStaticConstructorId,
        r#struct::Struct, type_handle::NonGenericTypeHandle,
    },
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::resource::ResourceManager,
};

pub mod cpu;
pub mod resource;

#[cfg(test)]
mod tests;

#[derive(ThreadSafe, Getters)]
#[getset(get = "pub")]
pub struct VirtualMachine {
    pub(crate) assembly_manager: AssemblyManager,
    resource_manager: ResourceManager,
    #[getset(skip)]
    #[allow(clippy::vec_box)]
    // cSpell:disable-next-line
    central_processing_units: RwLock<Vec<Box<CPU>>>,
    #[getset(skip)]
    cpu_for_static: RwLock<Box<CPU>>,

    #[getset(skip)]
    pub(crate) class_static_map: RwLock<HashMap<NonNull<Class>, ManagedReference<Class>>>,
    #[getset(skip)]
    #[allow(clippy::type_complexity)]
    pub(crate) struct_static_map: RwLock<HashMap<NonNull<Struct>, (NonNull<u8>, Layout)>>,
}

#[derive(Clone, Copy)]
pub enum CpuID {
    StaticCPU,
    Common(NonZero<usize>),
}

impl CpuID {
    pub fn as_global_cpu(&self) -> Option<MappedRwLockReadGuard<'static, CPU>> {
        global_vm().get_cpu(*self)
    }
}

impl VirtualMachine {
    pub fn construct_in(this: NonNull<Self>) {
        unsafe {
            this.byte_add(offset_of!(VirtualMachine, central_processing_units))
                .cast::<RwLock<Vec<Box<CPU>>>>()
                .write(RwLock::new(Vec::new()));
            this.byte_add(offset_of!(Self, cpu_for_static))
                .cast::<RwLock<Box<CPU>>>()
                .write(RwLock::new(Box::from_non_null(
                    CPU::new(this).as_non_null_ptr(),
                )));
            this.byte_add(offset_of!(Self, class_static_map))
                .cast::<RwLock<HashMap<NonNull<Class>, ManagedReference<Class>>>>()
                .write(RwLock::new(HashMap::new()));
            this.byte_add(offset_of!(Self, struct_static_map))
                .cast::<RwLock<HashMap<NonNull<Struct>, ManagedReference<Struct>>>>()
                .write(RwLock::new(HashMap::new()));
        }

        AssemblyManager::construct_in(this);
    }

    pub fn new() -> Unique<Self> {
        let this = alloc_type::<Self, _>(&std::alloc::Global).unwrap();

        Self::construct_in(this);

        Unique::from_non_null(this)
    }

    pub fn cpu_for_static(&self) -> MappedRwLockReadGuard<'_, CPU> {
        RwLockReadGuard::map(self.cpu_for_static.read().unwrap(), |x| &**x)
    }

    #[must_use]
    pub fn add_cpu(&self) -> CpuID {
        let mut central_processing_units = self.central_processing_units.write().unwrap();
        let index = central_processing_units.len();
        central_processing_units.push(unsafe {
            Box::from_non_null(CPU::new(NonNull::from_ref(self)).as_non_null_ptr())
        });
        CpuID::Common(unsafe { NonZero::new_unchecked(index + 1) })
    }

    pub fn get_cpu(&self, index: CpuID) -> Option<MappedRwLockReadGuard<'_, CPU>> {
        match index {
            CpuID::StaticCPU => Some(self.cpu_for_static()),
            CpuID::Common(index) => {
                RwLockReadGuard::filter_map(self.central_processing_units.read().unwrap(), |x| {
                    x.get(index.get() - 1).map(|x| &**x)
                })
                .ok()
            }
        }
    }

    pub fn load_class_static(&self, class: NonNull<Class>) -> ManagedReference<Class> {
        if let Some(v) = {
            let x = self.class_static_map.read().unwrap();
            x.get(&class).copied()
        } {
            return v;
        }
        let cpu = self.cpu_for_static();
        let obj = ManagedReference::<Class>::common_alloc(
            &cpu,
            unsafe { *class.as_ref().method_table() },
            true,
        );
        {
            let mut x = self.class_static_map.write().unwrap();
            x.insert(class, obj);
        }
        let sctor = unsafe {
            class
                .as_ref()
                .method_table_ref()
                .get_method(class.as_ref().__get_static_constructor_id())
                .unwrap()
        };

        unsafe {
            sctor.as_ref().typed_res_call::<()>(&cpu, None, &[]);
        }

        obj
    }

    pub fn load_struct_static(&self, s: NonNull<Struct>) -> (NonNull<u8>, Layout) {
        if let Some(v) = {
            let x = self.struct_static_map.read().unwrap();
            x.get(&s).copied()
        } {
            return v;
        }
        let cpu = self.cpu_for_static();
        let mt = unsafe { s.as_ref().method_table_ref() };
        let obj_layout = mt.static_layout(Default::default());
        let obj = Global.allocate(obj_layout).unwrap();
        {
            let mut x = self.struct_static_map.write().unwrap();
            x.insert(s, (obj.as_non_null_ptr(), obj_layout));
        }
        let sctor = unsafe {
            s.as_ref()
                .method_table_ref()
                .get_method(s.as_ref().__get_static_constructor_id())
                .unwrap()
        };

        unsafe {
            sctor.as_ref().typed_res_call::<()>(&cpu, None, &[]);
        }

        (obj.as_non_null_ptr(), obj_layout)
    }

    pub fn get_static_field(
        &self,
        ty: NonGenericTypeHandle,
        field: u32,
    ) -> Option<(NonNull<u8>, Layout)> {
        match ty {
            NonGenericTypeHandle::Class(class) => {
                let static_map = self.class_static_map.read().unwrap();
                let obj = if let Some(obj) = static_map.get(&class) {
                    *obj
                } else {
                    drop(static_map);
                    let mut static_map = self.class_static_map.write().unwrap();
                    let static_cpu = self.cpu_for_static();
                    let obj = ManagedReference::<Class>::common_alloc(
                        &static_cpu,
                        unsafe { *class.as_ref().method_table() },
                        true,
                    );
                    debug_assert!(obj.header().is_none_or(|x| x.is_static()));
                    static_map.insert(class, obj);
                    drop(static_map);
                    let sctor =
                        unsafe { class.as_ref().method_table_ref() }.get_static_constructor();
                    unsafe {
                        sctor.as_ref().typed_res_call::<()>(&static_cpu, None, &[]);
                    }

                    obj
                };
                debug_assert!(obj.header().is_none_or(|x| x.is_static()));
                obj.const_access::<FieldAccessor<_>>()
                    .field(field, Default::default())
            }
            NonGenericTypeHandle::Struct(s) => {
                let static_map = self.struct_static_map.read().unwrap();
                let (obj_p, _) = if let Some(x) = static_map.get(&s) {
                    *x
                } else {
                    drop(static_map);
                    let mut static_map = self.struct_static_map.write().unwrap();
                    let static_cpu = self.cpu_for_static();
                    let mt = unsafe { s.as_ref().method_table_ref() };
                    let obj_layout = mt.static_layout(Default::default());
                    let obj_p = std::alloc::Global
                        .allocate_zeroed(obj_layout)
                        .unwrap()
                        .as_non_null_ptr();
                    static_map.insert(s, (obj_p, obj_layout));
                    drop(static_map);
                    let sctor = mt.get_static_constructor();
                    unsafe {
                        sctor.as_ref().typed_res_call::<()>(&static_cpu, None, &[]);
                    }
                    (obj_p, obj_layout)
                };
                unsafe { s.as_ref().method_table_ref() }
                    .static_field_mem_info(field, Default::default(), Default::default())
                    .map(|x| (unsafe { obj_p.byte_add(x.offset).cast() }, x.layout))
            }
        }
    }
}

#[used]
#[unsafe(no_mangle)]
/* cSpell: disable-next-line */
static mut G_RUNTIM: MaybeUninit<VirtualMachine> = MaybeUninit::zeroed();
static ENSURE_VM_INIT: Once = Once::new();

#[inline(always)]
pub const unsafe fn global_vm_unchecked() -> &'static mut VirtualMachine {
    /* cSpell: disable-next-line */
    unsafe { G_RUNTIM.assume_init_mut() }
}

#[inline(always)]
pub fn global_vm() -> &'static mut VirtualMachine {
    if !ENSURE_VM_INIT.is_completed() {
        std::hint::cold_path();
        EnsureVirtualMachineInitialized();
    }
    unsafe { global_vm_unchecked() }
}

#[unsafe(no_mangle)]
#[cold]
pub extern "C" fn EnsureVirtualMachineInitialized() {
    ENSURE_VM_INIT.call_once(|| unsafe {
        /* cSpell: disable */
        let rt_ptr = G_RUNTIM.as_mut_ptr();
        VirtualMachine::construct_in(NonNull::new_unchecked(rt_ptr));
        /* cSpell: enable */
    });
}
