use std::{
    ptr::NonNull,
    sync::{MappedRwLockReadGuard, PoisonError, RwLock, RwLockReadGuard},
};

use global::StringName;

use crate::{stdlib::CoreTypeId, type_system::assembly::Assembly, virtual_machine::VirtualMachine};

use super::type_handle::NonGenericTypeHandle;

mod load_binary;

pub use load_binary::TypeLoadState;

pub struct AssemblyManager {
    #[allow(dead_code)]
    vm: NonNull<VirtualMachine>,

    #[allow(clippy::vec_box)]
    assemblies: RwLock<Vec<Box<Assembly>>>,
}

impl AssemblyManager {
    pub fn construct_in(mut vm: NonNull<VirtualMachine>) {
        unsafe {
            vm.as_mut().assembly_manager = Self {
                vm,
                assemblies: RwLock::new(Vec::new()),
            };
        }

        let this = unsafe { vm.as_ref().assembly_manager() };
        this.load_stdlib();
    }

    /// Just call [`crate::stdlib::load_stdlib`]
    #[inline(always)]
    #[track_caller]
    pub fn load_stdlib(&self) {
        crate::stdlib::load_stdlib(self);
    }
}

type ReadAssembliesPoisonError<'a> = PoisonError<RwLockReadGuard<'a, Vec<Box<Assembly>>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssemblyRef {
    Name(StringName),
    Id(usize),
}

impl PartialEq<usize> for AssemblyRef {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Self::Id(id) => id.eq(other),
            _ => false,
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
        let mut assemblies = self.assemblies.write().unwrap();
        assemblies.push(assembly);
        assemblies.len() - 1
    }

    /// It assumes that core assembly exists.
    pub fn get_core_assembly<'a>(
        &'a self,
    ) -> Result<MappedRwLockReadGuard<'a, Assembly>, ReadAssembliesPoisonError<'a>> {
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
        self.assemblies.read().map(|g| {
            RwLockReadGuard::filter_map(g, |x| x.first().filter(filter).map(map))
                .ok()
                .expect("Core Assembly (aka. `!`) is not found")
        })
    }

    /// It assumes that core assembly exists.
    pub fn get_core_type(&self, core_type_id: CoreTypeId) -> NonGenericTypeHandle {
        let assembly = self.get_core_assembly().unwrap();
        *assembly
            .get_type_handle(core_type_id as u32)
            .unwrap()
            .unwrap()
    }

    pub fn get_assembly_by_name<'a>(
        &'a self,
        name: &str,
    ) -> Result<Option<MappedRwLockReadGuard<'a, Assembly>>, ReadAssembliesPoisonError<'a>> {
        self.assemblies.read().map(|guard| {
            RwLockReadGuard::filter_map(guard, |x| {
                x.iter().find(|x| x.name.as_str().eq(name)).map(|x| &**x)
            })
            .ok()
        })
    }

    pub fn get_assembly<'a>(
        &'a self,
        id: usize,
    ) -> Result<Option<MappedRwLockReadGuard<'a, Assembly>>, ReadAssembliesPoisonError<'a>> {
        self.assemblies
            .read()
            .map(|guard| RwLockReadGuard::filter_map(guard, |x| x.get(id).map(|x| &**x)).ok())
    }

    pub fn get_assembly_by_ref<'a>(
        &'a self,
        r: &AssemblyRef,
    ) -> Result<Option<MappedRwLockReadGuard<'a, Assembly>>, ReadAssembliesPoisonError<'a>> {
        match r {
            AssemblyRef::Name(name) => self.get_assembly_by_name(name.as_str()),
            AssemblyRef::Id(id) => self.get_assembly(*id),
        }
    }
}
