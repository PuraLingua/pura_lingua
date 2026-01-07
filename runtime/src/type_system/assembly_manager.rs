use std::{
    ptr::NonNull,
    sync::{MappedRwLockReadGuard, PoisonError, RwLock, RwLockReadGuard},
};

use global::IndexMap;

use crate::{stdlib::CoreTypeId, type_system::assembly::Assembly, virtual_machine::VirtualMachine};

use super::type_handle::NonGenericTypeHandle;

pub struct AssemblyManager {
    #[allow(dead_code)]
    vm: NonNull<VirtualMachine>,

    assemblies: RwLock<IndexMap<String, Box<Assembly>>>,
}

impl AssemblyManager {
    pub fn construct_in(mut vm: NonNull<VirtualMachine>) {
        unsafe {
            vm.as_mut().assembly_manager = Self {
                vm,
                assemblies: RwLock::new(IndexMap::new()),
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

type ReadAssembliesPoisonError<'a> =
    PoisonError<RwLockReadGuard<'a, IndexMap<String, Box<Assembly>>>>;

impl AssemblyManager {
    pub fn add_assembly(&self, mut assembly: Box<Assembly>) {
        assembly.manager = NonNull::from_ref(self);
        let mut assemblies = self.assemblies.write().unwrap();
        assemblies.insert(assembly.name.as_str().to_owned(), assembly);
    }

    /// It assumes that core assembly exists.
    pub fn get_core_assembly<'a>(
        &'a self,
    ) -> Result<MappedRwLockReadGuard<'a, Assembly>, ReadAssembliesPoisonError<'a>> {
        if self.assemblies.try_read().is_err() {
            println!("Lock busy");
        }
        self.assemblies.read().map(|g| {
            RwLockReadGuard::filter_map(g, |x| x.get("!").filter(|x| x.is_core).map(|x| &**x))
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
        self.assemblies
            .read()
            .map(|guard| RwLockReadGuard::filter_map(guard, |x| x.get(name).map(|x| &**x)).ok())
    }
}
