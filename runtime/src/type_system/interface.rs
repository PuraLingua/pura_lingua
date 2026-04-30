use std::{
    alloc::{Allocator, Layout},
    mem::offset_of,
    ptr::NonNull,
    sync::MappedRwLockReadGuard,
};

use global::{
    attrs::TypeAttr,
    getset::{Getters, MutGetters},
};

use crate::{
    memory::OwnedPtr,
    type_system::{
        assembly::Assembly, class::Class, generics::GenericBounds, method::Method,
        method_table::MethodTable, type_handle::MaybeUnloadedTypeHandle,
    },
    value::managed_reference::ManagedReference,
};

#[derive(Getters, MutGetters, derive_more::Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Interface {
    assembly: NonNull<Assembly>,
    generic: Option<NonNull<Interface>>,

    name: Box<str>,
    attr: TypeAttr,

    /// They should be Interface
    required_interfaces: Vec<MaybeUnloadedTypeHandle>,

    #[getset(skip)]
    pub(crate) method_table: NonNull<MethodTable<Self>>,

    generic_instances: Vec<NonNull<Self>>,
    generic_bounds: Option<NonNull<[GenericBounds]>>,
    type_vars: Option<Box<[MaybeUnloadedTypeHandle]>>,
}

impl Interface {
    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> NonNull<Self> {
        for has_instantiated in self.generic_instances.iter() {
            if unsafe { has_instantiated.as_ref() }
                .type_vars
                .as_deref()
                .is_some_and(|x| x.eq(type_vars))
            {
                return *has_instantiated;
            }
        }

        let instantiated = Box::new(Self {
            assembly: self.assembly,
            generic: Some(NonNull::from_ref(self)),

            name: self.name.clone(),
            attr: self.attr,

            required_interfaces: self.required_interfaces.clone(),

            method_table: MethodTable::dup(self.method_table),

            generic_instances: Vec::new(),
            generic_bounds: None,
            type_vars: Some(Box::clone_from_ref(type_vars)),
        });

        let instantiated = Box::into_non_null(instantiated);

        unsafe {
            let mut mt = instantiated.as_ref().method_table;
            mt.as_mut().ty = instantiated;
            NonNull::from_ref(self)
                .byte_add(offset_of!(Self, generic_instances))
                .cast::<Vec<NonNull<Self>>>()
                .as_mut()
                .push(instantiated);
        }

        instantiated
    }

    /// The NonNull passed to mt_generator is always valid to be cast to &Self
    #[allow(clippy::too_many_arguments)]
    pub fn new<F: FnOnce(NonNull<Self>) -> NonNull<MethodTable<Self>>>(
        assembly: NonNull<Assembly>,

        name: String,
        attr: TypeAttr,

        required_interfaces: Vec<MaybeUnloadedTypeHandle>,

        mt_generator: F,

        generic_bounds: Option<Vec<GenericBounds>>,
    ) -> OwnedPtr<Self> {
        let this = Box::new(Self {
            assembly,
            generic: None,

            name: name.into_boxed_str(),
            attr,

            required_interfaces,

            // Methods are initialized afterwards
            method_table: NonNull::dangling(),

            generic_instances: Vec::new(),

            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,
        });

        let mut this = Box::into_non_null(this);

        unsafe {
            let mt = mt_generator(this);
            let this_m = this.as_mut();
            this_m.method_table = mt;
        }

        OwnedPtr::from_non_null(this)
    }
}

impl Interface {
    pub const fn assembly_ref(&self) -> &Assembly {
        unsafe { self.assembly.as_ref() }
    }
    pub const fn method_table_ref(&self) -> &MethodTable<Self> {
        unsafe { self.method_table.as_ref() }
    }
    pub fn is_generic(&self) -> bool {
        self.generic.is_none()
    }

    pub const fn val_layout(&self) -> Layout {
        Layout::new::<ManagedReference<Class>>()
    }

    pub fn val_libffi_type(&self) -> libffi::middle::Type {
        libffi::middle::Type::pointer()
    }

    pub fn get_method(&self, id: u32) -> Option<MappedRwLockReadGuard<'_, NonNull<Method<Self>>>> {
        self.method_table_ref().get_method(id)
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        if self.generic.is_none() {
            self.generic_bounds.inspect(|x| {
                unsafe {
                    x.drop_in_place();
                    std::alloc::Global
                        .deallocate(x.cast(), Layout::for_value_raw(x.as_ptr().cast_const()));
                };
            });
            for g in &self.generic_instances {
                unsafe {
                    g.drop_in_place();
                    std::alloc::Global.deallocate(g.cast(), Layout::new::<Interface>());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct InterfaceImplementation {
    pub target: MaybeUnloadedTypeHandle,
    pub map: Vec<u32>,
}
