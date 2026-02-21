use std::alloc::{Allocator, Layout};
use std::assert_matches;
use std::cell::Cell;
use std::mem::offset_of;
use std::ptr::{NonNull, Unique};
use std::sync::MappedRwLockReadGuard;

use either::Either;
use global::attrs::TypeAttr;
use global::getset::{Getters, MutGetters};

use crate::type_system::{
    assembly::Assembly, field::Field, generics::GenericBounds, method_table::MethodTable,
    type_handle::MaybeUnloadedTypeHandle,
};
use crate::value::managed_reference::ManagedReference;

use super::assembly_manager::TypeLoadState;
use super::method::Method;
use super::type_ref::TypeRef;

#[derive(Clone, Copy)]
pub(crate) union Parent {
    pub(crate) loaded: NonNull<Class>,
    pub(crate) unloaded: NonNull<TypeRef>,
}

#[derive(Getters, MutGetters, derive_more::Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Class {
    assembly: NonNull<Assembly>,
    generic: Option<NonNull<Class>>,
    pub(crate) load_state: TypeLoadState,
    main: Option<u32>,

    name: Box<str>,
    attr: TypeAttr,

    #[getset(skip)]
    #[debug(
        "{}",
        m_parent.map(|x| if self.load_state == TypeLoadState::Finished {
                format!("{:#?}", unsafe { &x.loaded })
            } else {
                format!("{:#?}", unsafe { &x.unloaded })
            }
        ).unwrap_or("None".to_owned()),
    )]
    pub(crate) m_parent: Option<Parent>,

    pub(crate) method_table: NonNull<MethodTable<Self>>,
    fields: Vec<Field>,
    sctor: u32,

    generic_instances: Vec<NonNull<Class>>,
    generic_bounds: Option<NonNull<[GenericBounds]>>,
    type_vars: Option<Box<[MaybeUnloadedTypeHandle]>>,
}

impl Class {
    pub const fn parent(&self) -> Option<NonNull<Self>> {
        const fn map(x: Parent) -> NonNull<Class> {
            unsafe { x.loaded }
        }
        self.m_parent.map(map)
    }
}

impl Class {
    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> NonNull<Self> {
        assert_matches!(self.load_state, TypeLoadState::Finished);
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
            load_state: TypeLoadState::Finished,
            main: self.main,

            name: self.name.clone(),
            attr: self.attr,

            m_parent: self.m_parent.map(|parent| unsafe {
                Parent {
                    loaded: parent.loaded.as_ref().instantiate(type_vars),
                }
            }),

            method_table: MethodTable::dup(self.method_table),
            fields: self
                .fields
                .iter()
                .map(|x| Field {
                    name: x.name.clone(),
                    attr: x.attr,
                    ty: x.ty.clone(),
                    cached_layout: Cell::new(None),
                    cached_offset: Cell::new(None),
                    cached_static_offset: Cell::new(None),
                })
                .collect(),
            sctor: self.sctor,

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

        parent: Option<NonNull<Class>>,

        mt_generator: F,
        mut fields: Vec<Field>,
        sctor: Option<u32>,

        generic_bounds: Option<Vec<GenericBounds>>,
    ) -> Unique<Self> {
        if let Some(parent) = parent.as_ref().map(|x| unsafe { x.as_ref() }) {
            let mut current_fields = fields;
            fields = parent.fields.clone();
            fields.append(&mut current_fields);
        }
        let this = Box::new(Self {
            assembly,
            generic: None,
            load_state: TypeLoadState::Finished,
            main: None,

            name: name.into_boxed_str(),
            attr,

            m_parent: parent.map(|x| Parent { loaded: x }),

            // MethodTable is initialized afterwards
            method_table: NonNull::dangling(),
            fields,
            sctor: 0,

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
            this_m.sctor = sctor.unwrap_or_else(|| {
                mt.as_ref()
                    .find_last_method_by_name_ret_id(".sctor")
                    .unwrap()
            });
            this_m.method_table = mt;
        }

        Unique::from_non_null(this)
    }

    /// The NonNull passed to mt_generator is always valid to be cast to &Self
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_binary<F: FnOnce(NonNull<Self>) -> NonNull<MethodTable<Self>>>(
        assembly: NonNull<Assembly>,

        name: String,
        attr: TypeAttr,

        parent: Option<Either<NonNull<Class>, TypeRef>>,

        mt_generator: F,
        fields: Vec<Field>,
        sctor: Option<u32>,

        generic_bounds: Option<Vec<GenericBounds>>,
    ) -> Unique<Self> {
        let mut load_state = TypeLoadState::Finished;
        let parent = parent.map(|x| match x {
            Either::Left(loaded) => Parent { loaded },
            Either::Right(unloaded) => {
                load_state = TypeLoadState::Loading;
                Parent {
                    unloaded: Box::into_non_null(Box::new(unloaded)),
                }
            }
        });
        let this = Box::new(Self {
            assembly,
            generic: None,
            load_state,
            main: None,

            name: name.into_boxed_str(),
            attr,

            m_parent: parent,

            // MethodTable is initialized afterwards
            method_table: NonNull::dangling(),
            fields,
            sctor: 0,

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
            if load_state == TypeLoadState::Finished {
                this_m.sctor = sctor.unwrap_or_else(|| {
                    mt.as_ref()
                        .find_last_method_by_name_ret_id(".sctor")
                        .unwrap()
                });
            }

            this_m.method_table = mt;
        }

        Unique::from_non_null(this)
    }

    pub(crate) fn rediscover_sctor(&mut self, hint: Option<u32>) {
        self.sctor = hint.unwrap_or_else(|| unsafe {
            self.method_table
                .as_ref()
                .find_last_method_by_name_ret_id(".sctor")
                .unwrap()
        });
    }
}

impl Class {
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
        Layout::new::<ManagedReference<Self>>()
    }

    pub fn val_libffi_type(&self) -> libffi::middle::Type {
        libffi::middle::Type::pointer()
    }

    pub fn get_method(&self, id: u32) -> Option<MappedRwLockReadGuard<'_, NonNull<Method<Self>>>> {
        self.method_table_ref().get_method(id)
    }
}

impl Drop for Class {
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
                    std::alloc::Global.deallocate(g.cast(), Layout::new::<Class>());
                }
            }
        }
    }
}
