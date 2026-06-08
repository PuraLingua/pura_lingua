use std::{
    ptr::NonNull,
    sync::nonpoison::{MappedRwLockReadGuard, RwLock, RwLockReadGuard},
};

use global::StringName;

use crate::{stdlib::CoreTypeId, type_system::assembly::Assembly, virtual_machine::VirtualMachine};

use super::type_handle::NonGenericTypeHandle;

mod load_binary;

pub use load_binary::{AtomicTypeLoadState, TypeLoadState};

pub struct AssemblyManager {
    #[allow(dead_code)]
    vm: NonNull<VirtualMachine>,

    #[allow(clippy::vec_box)]
    assemblies: RwLock<Vec<Box<Assembly>>>,
}

impl AssemblyManager {
    pub fn construct_in(vm: NonNull<VirtualMachine>) {
        unsafe {
            VirtualMachine::write_assembly_manager(
                vm,
                Self {
                    vm,
                    assemblies: RwLock::new(Vec::new()),
                },
            );
        }

        let this: &Self = unsafe { vm.as_ref().assembly_manager() };
        this.load_stdlib();
    }

    /// Just call [`crate::stdlib::load_stdlib`]
    #[inline(always)]
    #[track_caller]
    pub fn load_stdlib(&self) {
        crate::stdlib::load_stdlib(self);
    }

    pub const fn vm_ref(&self) -> &VirtualMachine {
        unsafe { self.vm.as_ref() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AssemblyRef {
    Name(StringName) = 0,
    Id(usize) = 1,
}

impl PartialEq<usize> for AssemblyRef {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Self::Id(id) => id.eq(other),
            _ => false,
        }
    }
}

impl std::fmt::Display for AssemblyRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyRef::Name(string_name) => f.write_str(string_name.as_str()),
            AssemblyRef::Id(id) => <_ as std::fmt::Display>::fmt(id, f),
        }
    }
}

impl AssemblyRef {
    pub const CORE: Self = Self::Id(0);
}

impl AssemblyManager {
    /// Returns id of the assembly
    pub fn add_assembly(&self, mut assembly: Box<Assembly>) -> usize {
        assembly.manager = NonNull::from_ref(self);
        let mut assemblies = self.assemblies.write();
        assemblies.push(assembly);
        assemblies.len() - 1
    }

    /// It assumes that core assembly exists.
    pub fn get_core_assembly<'a>(&'a self) -> MappedRwLockReadGuard<'a, Assembly> {
        if self.assemblies.try_read().is_err() {
            println!("Lock busy");
        }
        cfg_select! {
            debug_assertions => {
                #[inline(always)]
                #[allow(clippy::borrowed_box)]
                fn filter(assembly: &&Box<Assembly>) -> bool {
                    assembly.is_core
                }
            }
            _ => {
                #[inline(always)]
                #[allow(clippy::borrowed_box)]
                const fn filter(assembly: &&Box<Assembly>) -> bool { true }
            }
        }

        #[inline(always)]
        #[allow(clippy::borrowed_box)]
        fn map(x: &Box<Assembly>) -> &Assembly {
            x
        }
        RwLockReadGuard::filter_map(self.assemblies.read(), |x| {
            x.first().filter(filter).map(map)
        })
        .ok()
        .unwrap()
    }

    /// It assumes that core assembly exists.
    pub fn get_core_type(&self, core_type_id: CoreTypeId) -> NonGenericTypeHandle {
        let assembly = self.get_core_assembly();
        assembly.get_type_handle(core_type_id as u32).unwrap()
    }

    pub fn get_assembly_by_name<'a>(
        &'a self,
        name: &widestring::Utf16Str,
    ) -> Option<MappedRwLockReadGuard<'a, Assembly>> {
        RwLockReadGuard::filter_map(self.assemblies.read(), |x: &Vec<Box<Assembly>>| {
            x.iter().find(|x| (&*x.name).eq(name)).map(|x| &**x)
        })
        .ok()
    }

    pub fn get_assembly<'a>(&'a self, id: usize) -> Option<MappedRwLockReadGuard<'a, Assembly>> {
        RwLockReadGuard::filter_map(self.assemblies.read(), |x| x.get(id).map(|x| &**x)).ok()
    }

    pub fn get_assembly_by_ref<'a>(
        &'a self,
        r: &AssemblyRef,
    ) -> Option<MappedRwLockReadGuard<'a, Assembly>> {
        match r {
            AssemblyRef::Name(name) => {
                self.get_assembly_by_name(&widestring::Utf16String::from_str(name.as_str()))
            }
            AssemblyRef::Id(id) => self.get_assembly(*id),
        }
    }
}
