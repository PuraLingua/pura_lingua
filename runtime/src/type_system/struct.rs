use std::{
    alloc::{Allocator, Layout},
    cell::Cell,
    mem::offset_of,
    ops::RangeBounds,
    ptr::NonNull,
    sync::MappedRwLockReadGuard,
};

use global::{
    attrs::TypeAttr,
    getset::{Getters, MutGetters},
    non_purus_call_configuration::NonPurusCallType,
};
use stdlib_header::CoreTypeId;

use crate::{
    memory::{GetLayoutOptions, OwnedPtr},
    stdlib::CoreTypeIdExt as _,
    type_system::{
        assembly::Assembly,
        field::Field,
        generics::{GenericBounds, GenericCountRequirement},
        method_table::MethodTable,
        type_handle::MaybeUnloadedTypeHandle,
    },
};

use super::method::Method;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Struct {
    assembly: NonNull<Assembly>,
    generic: Option<NonNull<Struct>>,

    name: Box<str>,
    attr: TypeAttr,
    generic_count_requirement: GenericCountRequirement,

    // Note that Struct does not have parents
    pub(crate) method_table: NonNull<MethodTable<Self>>,
    fields: Vec<Field>,
    sctor: u32,

    generic_instances: Vec<NonNull<Struct>>,
    generic_bounds: Option<NonNull<[GenericBounds]>>,
    type_vars: Option<Box<[MaybeUnloadedTypeHandle]>>,
}

impl Struct {
    /// The NonNull passed to mt_generator is always valid to be cast to &Self
    pub fn new<F: FnOnce(NonNull<Self>) -> NonNull<MethodTable<Self>>>(
        assembly: NonNull<Assembly>,

        name: String,
        attr: TypeAttr,
        generic_count_requirement: GenericCountRequirement,

        mt_generator: F,
        fields: Vec<Field>,
        sctor: Option<u32>,

        generic_bounds: Option<Vec<GenericBounds>>,
    ) -> OwnedPtr<Self> {
        let this = Box::new(Self {
            assembly,
            generic: None,

            name: name.into_boxed_str(),
            attr,
            generic_count_requirement,

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

        OwnedPtr::from_non_null(this)
    }

    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> NonNull<Self> {
        assert!(
            self.generic_count_requirement
                .contains(&(type_vars.len() as u32))
        );
        for has_instantiated in self.generic_instances.iter() {
            if unsafe { has_instantiated.as_ref() }
                .type_vars
                .as_deref()
                .is_some_and(|x| x.eq(type_vars))
            {
                return *has_instantiated;
            }
        }
        if self
            .method_table_ref()
            .get_core_type_id()
            .is_some_and(|x| x == CoreTypeId::System_Tuple)
        {
            let mut this = Self::new(
                self.assembly,
                self.name.clone().into_string(),
                self.attr,
                self.generic_count_requirement,
                |x| {
                    let mut mt = MethodTable::dup(self.method_table);
                    unsafe {
                        mt.as_mut().ty = x;
                    }
                    mt
                },
                type_vars
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| {
                        Field::new(
                            index.to_string(),
                            global::attr!(field Public {}),
                            ty.clone(),
                        )
                    })
                    .collect(),
                Some(self.sctor),
                None,
            )
            .as_non_null_ptr();
            unsafe {
                this.as_mut().type_vars = Some(Box::clone_from_ref(type_vars));

                NonNull::from_ref(self)
                    .byte_add(offset_of!(Self, generic_instances))
                    .cast::<Vec<NonNull<Self>>>()
                    .as_mut()
                    .push(this);
            }
            return this;
        }
        let instantiated = Box::new(Self {
            assembly: self.assembly,
            generic: Some(NonNull::from_ref(self)),

            name: self.name.clone(),
            attr: self.attr,
            generic_count_requirement: self.generic_count_requirement,

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
            instantiated
                .as_ref()
                .method_table
                .byte_add(offset_of!(MethodTable<Self>, ty))
                .cast::<NonNull<Self>>()
                .write(instantiated);
            NonNull::from_ref(self)
                .byte_add(offset_of!(Self, generic_instances))
                .cast::<Vec<NonNull<Self>>>()
                .as_mut()
                .push(instantiated);
        }

        instantiated
    }
}

impl Struct {
    pub const fn assembly_ref(&self) -> &Assembly {
        unsafe { self.assembly.as_ref() }
    }

    pub const fn method_table_ref(&self) -> &MethodTable<Self> {
        unsafe { self.method_table.as_ref() }
    }

    pub fn val_layout(&self) -> Layout {
        self.method_table_ref()
            .mem_layout(GetLayoutOptions::default())
    }

    pub fn val_libffi_type(&self) -> libffi::middle::Type {
        if let Some(core_type_id) = self.method_table_ref().get_core_type_id()
            && let Some(gotten_ty) = core_type_id.val_libffi_type()
        {
            return gotten_ty;
        }

        let mut builder = Vec::with_capacity(self.fields.len());
        for f in self.fields() {
            builder.push(f.libffi_type_with_type(self));
        }

        libffi::middle::Type::structure(builder)
    }

    pub fn non_purus_call_type(&self) -> NonPurusCallType {
        if let Some(core_type_id) = self.method_table_ref().get_core_type_id()
            && let Some(gotten_ty) = core_type_id.non_purus_call_type()
        {
            std::hint::cold_path(); // It should be handled by caller usually.
            return gotten_ty;
        }

        let mut builder = Vec::with_capacity(self.fields.len());
        for f in self.fields() {
            builder.push(f.non_purus_call_type_with_type(self));
        }

        NonPurusCallType::Structure(builder)
    }

    pub fn get_method(&self, id: u32) -> Option<MappedRwLockReadGuard<'_, NonNull<Method<Self>>>> {
        self.method_table_ref().get_method(id)
    }
}

impl Drop for Struct {
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
                    std::alloc::Global.deallocate(g.cast(), Layout::new::<Struct>());
                }
            }
        }
    }
}
