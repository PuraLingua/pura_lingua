use std::{
    alloc::{Allocator, Global, Layout},
    collections::HashMap,
    mem::MaybeUninit,
    pin::Pin,
    ptr::NonNull,
    sync::{
        Once,
        nonpoison::{RwLock, RwLockWriteGuard},
    },
};

use cpu::CPU;
use global::{ThreadSafe, getset::Getters};

use crate::{
    type_system::{
        assembly_manager::AssemblyManager, class::Class, get_traits::GetStaticConstructorId,
        r#struct::Struct, type_handle::NonGenericTypeHandle,
    },
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::{
        cpu_manager::{CPUManager, CpuID},
        resource::ResourceManager,
    },
};

pub mod cpu;
pub mod cpu_manager;
pub mod resource;

#[cfg(test)]
mod tests;

#[derive(ThreadSafe, Getters)]
#[getset(get = "pub")]
pub struct VirtualMachine {
    assembly_manager: AssemblyManager,
    resource_manager: ResourceManager,
    cpu_manager: CPUManager,

    #[getset(skip)]
    pub(crate) class_static_map: RwLock<HashMap<NonNull<Class>, ManagedReference<Class>>>,
    #[getset(skip)]
    #[allow(clippy::type_complexity)]
    pub(crate) struct_static_map: RwLock<HashMap<NonNull<Struct>, (NonNull<u8>, Layout)>>,
}

impl VirtualMachine {
    pub unsafe fn write_assembly_manager(this: NonNull<Self>, assembly_manager: AssemblyManager) {
        unsafe {
            this.byte_add(std::mem::offset_of!(Self, assembly_manager))
                .cast::<AssemblyManager>()
                .write(assembly_manager);
        }
    }
    pub fn construct_in(this: NonNull<Self>) {
        unsafe {
            this.write(VirtualMachine {
                #[expect(invalid_value, reason = "It will be init then")]
                assembly_manager: MaybeUninit::uninit().assume_init(),
                resource_manager: ResourceManager {},
                #[expect(invalid_value, reason = "It will be init then")]
                cpu_manager: MaybeUninit::uninit().assume_init(),
                class_static_map: RwLock::new(HashMap::new()),
                struct_static_map: RwLock::new(HashMap::new()),
            });
        }

        AssemblyManager::construct_in(this);
        CPUManager::construct_in(this);
    }

    pub fn new_in<A: Allocator>(allocator: A) -> Box<Self, A> {
        let this: NonNull<Self> = allocator.allocate(Layout::new::<Self>()).unwrap().cast();

        Self::construct_in(this);

        unsafe { Box::from_non_null_in(this, allocator) }
    }

    pub fn new() -> Box<Self> {
        Self::new_in(std::alloc::Global)
    }

    pub fn new_system() -> Box<Self, std::alloc::System> {
        Self::new_in(std::alloc::System)
    }

    pub fn cpu_for_static(&self) -> Pin<&RwLock<CPU>> {
        self.cpu_manager.cpu_for_static()
    }

    pub fn write_cpu_for_static<'a>(&'a self) -> RwLockWriteGuard<'a, CPU> {
        self.cpu_manager.write_cpu_for_static()
    }

    #[must_use]
    pub fn add_cpu(&self) -> CpuID {
        self.cpu_manager.add_cpu()
    }

    pub fn get_cpu(&self, index: CpuID) -> Option<Pin<&RwLock<CPU>>> {
        self.cpu_manager.get_cpu(index)
    }

    pub fn write_cpu<'a>(&'a self, index: CpuID) -> Option<RwLockWriteGuard<'a, CPU>> {
        self.get_cpu(index).map(Pin::get_ref).map(|x| x.write())
    }

    pub fn load_class_static(&self, class: NonNull<Class>) -> ManagedReference<Class> {
        if let Some(v) = {
            let x = self.class_static_map.read();
            x.get(&class).copied()
        } {
            return v;
        }
        let mut cpu = self.write_cpu_for_static();
        let obj = ManagedReference::<Class>::common_alloc(
            &mut cpu,
            unsafe { *class.as_ref().method_table() },
            true,
        );
        {
            let mut x = self.class_static_map.write();
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
            sctor.as_ref().typed_res_call::<()>(&mut cpu, None, &[]);
        }

        obj
    }

    pub fn load_struct_static(&self, s: NonNull<Struct>) -> (NonNull<u8>, Layout) {
        if let Some(v) = {
            let x = self.struct_static_map.read();
            x.get(&s).copied()
        } {
            return v;
        }
        let mut cpu = self.write_cpu_for_static();
        let mt = unsafe { s.as_ref().method_table_ref() };
        let obj_layout = mt.static_layout(Default::default());
        let obj = Global.allocate(obj_layout).unwrap();
        {
            let mut x = self.struct_static_map.write();
            x.insert(s, (obj.as_non_null_ptr(), obj_layout));
        }
        let sctor = unsafe {
            s.as_ref()
                .method_table_ref()
                .get_method(s.as_ref().__get_static_constructor_id())
                .unwrap()
        };

        unsafe {
            sctor.as_ref().typed_res_call::<()>(&mut cpu, None, &[]);
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
                let static_map = self.class_static_map.read();
                let obj = if let Some(obj) = static_map.get(&class) {
                    *obj
                } else {
                    drop(static_map);
                    let mut static_map = self.class_static_map.write();
                    let mut static_cpu = self.write_cpu_for_static();
                    let obj = ManagedReference::<Class>::common_alloc(
                        &mut static_cpu,
                        unsafe { *class.as_ref().method_table() },
                        true,
                    );
                    static_map.insert(class, obj);
                    drop(static_map);
                    let sctor =
                        unsafe { class.as_ref().method_table_ref() }.get_static_constructor();
                    unsafe {
                        sctor
                            .as_ref()
                            .typed_res_call::<()>(&mut static_cpu, None, &[]);
                    }

                    obj
                };
                debug_assert!(obj.header().is_none_or(|x| x.is_static()));
                obj.const_access::<FieldAccessor<_>>()
                    .field(field, Default::default())
            }
            NonGenericTypeHandle::Struct(s) => {
                let static_map = self.struct_static_map.read();
                let (obj_p, _) = if let Some(x) = static_map.get(&s) {
                    *x
                } else {
                    drop(static_map);
                    let mut static_map = self.struct_static_map.write();
                    let mut static_cpu = self.write_cpu_for_static();
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
                        sctor
                            .as_ref()
                            .typed_res_call::<()>(&mut static_cpu, None, &[]);
                    }
                    (obj_p, obj_layout)
                };
                unsafe { s.as_ref().method_table_ref() }
                    .static_field_mem_info(field, Default::default(), Default::default())
                    .map(|x| (unsafe { obj_p.byte_add(x.offset).cast() }, x.layout))
            }
            NonGenericTypeHandle::Interface(_) => None,
        }
    }
}

/* cSpell: disable-next-line */
static mut G_RUNTIME: MaybeUninit<VirtualMachine> = MaybeUninit::zeroed();
static VM_INIT: Once = Once::new();

#[inline(always)]
#[allow(static_mut_refs)]
pub const unsafe fn global_vm_unchecked() -> &'static mut VirtualMachine {
    unsafe { G_RUNTIME.assume_init_mut() }
}

#[inline(always)]
pub fn global_vm() -> &'static mut VirtualMachine {
    if !VM_INIT.is_completed() {
        std::hint::cold_path();
        EnsureGlobalVirtualMachineInitialized();
    }
    unsafe { global_vm_unchecked() }
}

#[inline(always)]
pub fn is_global_vm_init() -> bool {
    VM_INIT.is_completed()
}

#[allow(nonstandard_style)]
#[allow(static_mut_refs)]
pub fn EnsureGlobalVirtualMachineInitialized() {
    VM_INIT.call_once(|| unsafe {
        let rt_ptr = G_RUNTIME.as_mut_ptr();
        VirtualMachine::construct_in(NonNull::new_unchecked(rt_ptr));
    });
}
