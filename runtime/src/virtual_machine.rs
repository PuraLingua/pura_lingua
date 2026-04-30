use std::{
    alloc::{Allocator, Global, Layout},
    cell::SyncUnsafeCell,
    collections::HashMap,
    mem::MaybeUninit,
    num::NonZero,
    pin::Pin,
    ptr::NonNull,
    sync::{LockResult, Mutex, Once, RwLock, RwLockWriteGuard},
};

use cpu::CPU;
use global::{ThreadSafe, getset::Getters};

use crate::{
    memory::OwnedPtr,
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
    #[doc(hidden)]
    pub(crate) assembly_manager: AssemblyManager,
    resource_manager: ResourceManager,
    #[getset(skip)]
    cpu_lock: Mutex<()>,
    #[getset(skip)]
    #[allow(clippy::vec_box)]
    central_processing_units: SyncUnsafeCell<Vec<Pin<Box<RwLock<CPU>>>>>,
    #[getset(skip)]
    cpu_for_static: Pin<Box<RwLock<CPU>>>,

    #[getset(skip)]
    pub(crate) class_static_map: RwLock<HashMap<NonNull<Class>, ManagedReference<Class>>>,
    #[getset(skip)]
    #[allow(clippy::type_complexity)]
    pub(crate) struct_static_map: RwLock<HashMap<NonNull<Struct>, (NonNull<u8>, Layout)>>,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CpuID {
    StaticCPU = 0,
    Common(NonZero<usize>) = 1,
}

impl CpuID {
    pub fn new_global() -> Pin<&'static RwLock<CPU>> {
        global_vm().add_cpu().as_global_cpu().unwrap()
    }
    pub fn new_write_global() -> RwLockWriteGuard<'static, CPU> {
        global_vm().add_cpu().as_global_write_cpu().unwrap()
    }
    pub fn as_global_cpu(&self) -> Option<Pin<&'static RwLock<CPU>>> {
        global_vm().get_cpu(*self)
    }
    pub fn as_global_write_cpu(&self) -> Option<RwLockWriteGuard<'static, CPU>> {
        self.as_global_cpu()
            .map(Pin::get_ref)
            .map(|x| x.write().unwrap())
    }
}

impl VirtualMachine {
    pub fn construct_in(this: NonNull<Self>) {
        unsafe {
            this.write(VirtualMachine {
                #[expect(invalid_value, reason = "It will be init then")]
                assembly_manager: MaybeUninit::uninit().assume_init(),
                resource_manager: ResourceManager {},
                cpu_lock: Mutex::new(()),
                central_processing_units: SyncUnsafeCell::new(Vec::new()),
                cpu_for_static: CPU::new(this),
                class_static_map: RwLock::new(HashMap::new()),
                struct_static_map: RwLock::new(HashMap::new()),
            });
        }

        AssemblyManager::construct_in(this);
    }

    pub fn new_in<A: Allocator>(allocator: &A) -> OwnedPtr<Self> {
        let this = allocator.allocate(Layout::new::<Self>()).unwrap().cast();

        Self::construct_in(this);

        OwnedPtr::from_non_null(this)
    }

    pub fn new() -> OwnedPtr<Self> {
        Self::new_in(&std::alloc::Global)
    }

    pub fn new_system() -> OwnedPtr<Self> {
        Self::new_in(&std::alloc::System)
    }

    pub fn cpu_for_static(&self) -> Pin<&RwLock<CPU>> {
        Pin::as_ref(&self.cpu_for_static)
    }

    pub fn write_cpu_for_static<'a>(&'a self) -> LockResult<RwLockWriteGuard<'a, CPU>> {
        self.cpu_for_static.write()
    }

    #[must_use]
    pub fn add_cpu(&self) -> CpuID {
        let _guard = self.cpu_lock.lock().unwrap();
        let central_processing_units =
            unsafe { self.central_processing_units.get().as_mut_unchecked() };
        let index = central_processing_units.len();
        central_processing_units.push(CPU::new(NonNull::from_ref(self)));
        CpuID::Common(unsafe { NonZero::new_unchecked(index + 1) })
    }

    pub fn get_cpu(&self, index: CpuID) -> Option<Pin<&RwLock<CPU>>> {
        match index {
            CpuID::StaticCPU => Some(self.cpu_for_static()),
            CpuID::Common(index) => {
                let _guard = self.cpu_lock.lock().unwrap();
                let central_processing_units =
                    unsafe { self.central_processing_units.get().as_ref_unchecked() };
                central_processing_units
                    .get(index.get() - 1)
                    .map(|x| Pin::as_ref(x))
            }
        }
    }

    pub fn write_cpu<'a>(&'a self, index: CpuID) -> Option<LockResult<RwLockWriteGuard<'a, CPU>>> {
        self.get_cpu(index).map(Pin::get_ref).map(|x| x.write())
    }

    pub fn load_class_static(&self, class: NonNull<Class>) -> ManagedReference<Class> {
        if let Some(v) = {
            let x = self.class_static_map.read().unwrap();
            x.get(&class).copied()
        } {
            return v;
        }
        let mut cpu = self.write_cpu_for_static().unwrap();
        let obj = ManagedReference::<Class>::common_alloc(
            &mut cpu,
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
            sctor.as_ref().typed_res_call::<()>(&mut cpu, None, &[]);
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
        let mut cpu = self.write_cpu_for_static().unwrap();
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
                let static_map = self.class_static_map.read().unwrap();
                let obj = if let Some(obj) = static_map.get(&class) {
                    *obj
                } else {
                    drop(static_map);
                    let mut static_map = self.class_static_map.write().unwrap();
                    let mut static_cpu = self.write_cpu_for_static().unwrap();
                    println!("Initializing static for {}", unsafe {
                        class.as_ref().name()
                    });
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
                let static_map = self.struct_static_map.read().unwrap();
                let (obj_p, _) = if let Some(x) = static_map.get(&s) {
                    *x
                } else {
                    drop(static_map);
                    let mut static_map = self.struct_static_map.write().unwrap();
                    let mut static_cpu = self.write_cpu_for_static().unwrap();
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
pub fn EnsureGlobalVirtualMachineInitialized() {
    VM_INIT.call_once(|| unsafe {
        let rt_ptr = G_RUNTIME.as_mut_ptr();
        VirtualMachine::construct_in(NonNull::new_unchecked(rt_ptr));
    });
}
