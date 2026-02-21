use std::{alloc::Layout, cell::Cell, ptr::NonNull};

use global::{attrs::FieldAttr, getset::Getters};

use crate::{
    memory::GetLayoutOptions,
    type_system::type_handle::{MaybeUnloadedTypeHandle, TypeHandle},
};

use super::{
    assembly_manager::AssemblyManager,
    get_traits::{GetAssemblyRef, GetTypeVars},
    type_handle::{NonGenericTypeHandle, type_generic_resolver},
};

#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct Field {
    pub(crate) name: Box<str>,
    pub(crate) attr: FieldAttr,
    pub(crate) ty: MaybeUnloadedTypeHandle,

    #[getset(skip)]
    pub(crate) cached_layout: Cell<Option<Layout>>,
    #[getset(skip)]
    pub(crate) cached_offset: Cell<Option<usize>>,
    #[getset(skip)]
    pub(crate) cached_static_offset: Cell<Option<usize>>,
}

impl Field {
    pub fn new(name: String, attr: FieldAttr, ty: MaybeUnloadedTypeHandle) -> Self {
        Self {
            name: name.into_boxed_str(),
            attr,
            ty,
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
        match self.ty {
            MaybeUnloadedTypeHandle::Loaded(th) => th.val_layout(),
            MaybeUnloadedTypeHandle::Unloaded(_) => None,
        }
        .inspect(|x| {
            if !options.discard_calculated_layout {
                self.cached_layout.set(Some(*x));
            }
        })
    }

    fn load_type(&self, manager: &AssemblyManager) -> Option<TypeHandle> {
        self.ty.load(manager).inspect(|ty| unsafe {
            NonNull::from_ref(self).as_mut().ty = MaybeUnloadedTypeHandle::Loaded(*ty);
        })
    }

    fn get_type_with_generic_resolver<F: Fn(u32) -> TypeHandle>(
        &self,
        f: F,
    ) -> NonGenericTypeHandle {
        let mut ty = self.ty.clone().assume_init();
        loop {
            match ty.as_non_generic() {
                Some(ty) => return *ty,
                _ => {
                    let TypeHandle::Generic(g_index) = ty else {
                        unreachable!()
                    };

                    ty = f(g_index);
                }
            }
        }
    }

    pub fn get_type_with_type<T: GetAssemblyRef + GetTypeVars>(
        &self,
        ty: &T,
    ) -> NonGenericTypeHandle {
        self.get_type_with_generic_resolver(|g_index| type_generic_resolver(g_index, ty))
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

        if type_vars.is_none() && matches!(th, TypeHandle::Generic(_)) {
            unimplemented!()
        }

        let Some(type_vars) = type_vars.as_ref() else {
            unreachable!()
        };

        while let TypeHandle::Generic(g_index) = th {
            if let Some(t) = type_vars.get(g_index as usize) {
                th = t.load(ty.__get_assembly_ref().manager_ref()).unwrap();
            } else {
                break; // It leads to panicking at the unwrap method
            }
        }

        th.as_non_generic().unwrap().val_libffi_type()
    }
}
