use std::{alloc::Layout, cell::Cell};

use global::{attrs::FieldAttr, getset::Getters, non_purus_call_configuration::NonPurusCallType};

use crate::{
    memory::GetLayoutOptions,
    type_system::{
        cached_type_reference::CachedTypeReference,
        type_handle::{
            GenericUnresolvable, IGenericResolver, MaybeUnloadedTypeHandle, TypeGenericResolver,
            TypeHandle,
        },
    },
    utils::clone_utf16str,
};

use super::{
    assembly_manager::AssemblyManager,
    get_traits::{GetAssemblyRef, GetTypeVars},
    type_handle::NonGenericTypeHandle,
};

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct Field {
    pub(crate) name: Box<widestring::Utf16Str>,
    pub(crate) attr: FieldAttr,
    pub(crate) ty: CachedTypeReference,

    #[getset(skip)]
    pub(crate) cached_layout: Cell<Option<Layout>>,
    #[getset(skip)]
    pub(crate) cached_offset: Cell<Option<usize>>,
    #[getset(skip)]
    pub(crate) cached_static_offset: Cell<Option<usize>>,
}

impl Clone for Field {
    fn clone(&self) -> Self {
        Self {
            name: clone_utf16str(&self.name),
            attr: self.attr,
            ty: self.ty.clone(),
            cached_layout: Cell::new(None),
            cached_offset: Cell::new(None),
            cached_static_offset: Cell::new(None),
        }
    }
}

impl Field {
    pub fn new(
        name: widestring::Utf16String,
        attr: FieldAttr,
        ty: MaybeUnloadedTypeHandle,
    ) -> Self {
        Self {
            name: name.into_boxed_utfstr(),
            attr,
            ty: CachedTypeReference::new(ty),
            cached_layout: Cell::new(None),
            cached_offset: Cell::new(None),
            cached_static_offset: Cell::new(None),
        }
    }
}

impl Field {
    pub fn layout(&self, options: GetLayoutOptions) -> Option<Layout> {
        if options.prefer_cached
            && let Some(layout) = self.cached_layout.get()
        {
            return Some(layout);
        }

        self.ty
            .to_handle()
            .and_then(|x| x.val_layout())
            .inspect(|x| {
                if !options.discard_calculated_layout {
                    self.cached_layout.set(Some(*x));
                }
            })
    }

    fn load_type(&self, manager: &AssemblyManager) -> Option<TypeHandle> {
        self.ty
            .get_with_generic_resolver(manager, &GenericUnresolvable)
    }

    fn get_type_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        resolver: &TResolver,
    ) -> NonGenericTypeHandle {
        let mut ty = self.ty.assume_init();
        loop {
            match ty.as_non_generic() {
                Some(ty) => return *ty,
                _ => {
                    ty = match ty {
                        TypeHandle::MethodGeneric(g_index) => {
                            resolver.resolve_method_generic(g_index)
                        }
                        TypeHandle::TypeGeneric(g_index) => resolver.resolve_type_generic(g_index),

                        _ => unreachable!(),
                    }
                    .unwrap();
                    if let Some(ty) = ty.into_non_generic() {
                        return ty;
                    }
                }
            }
        }
    }

    pub fn get_type_with_type<T: GetAssemblyRef + GetTypeVars>(
        &self,
        ty: &T,
    ) -> NonGenericTypeHandle {
        self.get_type_with_generic_resolver(TypeGenericResolver::new(ty))
    }

    pub fn layout_with_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        ty: &T,
        options: GetLayoutOptions,
    ) -> Layout {
        if options.prefer_cached
            && let Some(layout) = self.cached_layout.get()
        {
            return layout;
        }
        self.load_type(ty.__get_assembly_ref().manager_ref())
            .unwrap();

        let th = self.get_type_with_type(ty);

        let layout = th.val_layout();

        if !options.discard_calculated_layout {
            self.cached_layout.set(Some(layout));
        }

        layout
    }

    pub fn libffi_type_with_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        ty: &T,
    ) -> libffi::middle::Type {
        let mut th = self
            .load_type(ty.__get_assembly_ref().manager_ref())
            .unwrap();

        let type_vars = ty.__get_type_vars();

        if type_vars.is_none() && matches!(th, TypeHandle::TypeGeneric(_)) {
            unimplemented!()
        }

        let Some(type_vars) = type_vars.as_ref() else {
            unreachable!()
        };

        while let TypeHandle::TypeGeneric(g_index) = th {
            if let Some(t) = type_vars.get(g_index as usize) {
                th = (*t).into();
            } else {
                break; // It leads to panicking at the unwrap method
            }
        }

        th.as_non_generic().unwrap().val_libffi_type()
    }

    pub fn non_purus_call_type_with_type<T: GetTypeVars + GetAssemblyRef>(
        &self,
        ty: &T,
    ) -> NonPurusCallType {
        let mut th = self
            .load_type(ty.__get_assembly_ref().manager_ref())
            .unwrap();

        let type_vars = ty.__get_type_vars();

        if type_vars.is_none() && matches!(th, TypeHandle::TypeGeneric(_)) {
            unimplemented!()
        }

        let Some(type_vars) = type_vars.as_ref() else {
            return th.into_non_generic().unwrap().non_purus_call_type();
        };

        while let TypeHandle::TypeGeneric(g_index) = th {
            if let Some(&t) = type_vars.get(g_index as usize) {
                th = t.into();
            } else {
                break; // It leads to panicking at the unwrap method
            }
        }

        th.as_non_generic().unwrap().non_purus_call_type()
    }
}
