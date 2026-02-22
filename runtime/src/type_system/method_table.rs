use std::{
    alloc::Layout,
    cell::Cell,
    ptr::{NonNull, Unique},
    sync::{MappedRwLockReadGuard, RwLock, RwLockReadGuard},
};

use enumflags2::BitFlags;

use crate::{
    memory::{GetFieldOffsetOptions, GetLayoutOptions},
    stdlib::{CoreTypeId, CoreTypeIdConstExt as _},
    type_system::method::Method,
};

use super::{
    field::Field,
    get_traits::{
        GetAssemblyRef, GetFields, GetGeneric, GetMethodTableRef, GetParent,
        GetStaticConstructorId, GetTypeVars,
    },
    method::MethodRef,
    type_handle::NonGenericTypeHandle,
};

mod display;

#[repr(C)]
pub struct MethodTable<T> {
    pub(crate) ty: NonNull<T>,
    methods: RwLock<Vec<NonNull<Method<T>>>>,
    __override_methods: Vec<usize>,

    cached_layout: Cell<Option<Layout>>,
    cached_static_layout: Cell<Option<Layout>>,
}

impl<T> MethodTable<T> {
    #[inline]
    pub const fn ty_ref(&self) -> &T {
        unsafe { self.ty.as_ref() }
    }
    #[inline]
    pub const fn ty_mut(&mut self) -> &mut T {
        unsafe { self.ty.as_mut() }
    }
}

impl<T> MethodTable<T> {
    /// Clone it without any caches
    pub fn dup(p: NonNull<Self>) -> NonNull<Self> {
        unsafe {
            let mut this = Box::into_non_null(Box::new(Self {
                ty: p.as_ref().ty,
                methods: RwLock::new(Vec::new()),
                __override_methods: p.as_ref().__override_methods.clone(),
                cached_layout: Cell::new(None),
                cached_static_layout: Cell::new(None),
            }));
            let mut methods = p.as_ref().methods.get_cloned().unwrap();
            methods.iter_mut().for_each(|x| x.as_mut().mt = Some(this));
            this.as_mut().methods = RwLock::new(methods);
            this
        }
    }
}

impl<T> MethodTable<T> {
    pub fn get_method(&self, id: u32) -> Option<MappedRwLockReadGuard<'_, NonNull<Method<T>>>> {
        RwLockReadGuard::filter_map(self.methods.read().unwrap(), |x| x.get(id as usize)).ok()
    }
    pub fn list_method_signatures(&self) -> Vec<String> {
        let methods = self.methods.read().unwrap();
        methods
            .iter()
            .map(|x| unsafe { x.as_ref().display(BitFlags::all()).to_string() })
            .collect()
    }
    pub fn find_first_method_by_name<TName: ?Sized>(
        &self,
        name: &TName,
    ) -> Option<MappedRwLockReadGuard<'_, NonNull<Method<T>>>>
    where
        str: PartialEq<TName>,
    {
        RwLockReadGuard::filter_map(self.methods.read().unwrap(), |x| {
            x.iter()
                .find(|m| unsafe { m.as_ref().name().as_str().eq(name) })
        })
        .ok()
    }

    pub fn find_last_method_by_name_ret_id<TName: ?Sized>(&self, name: &TName) -> Option<u32>
    where
        str: PartialEq<TName>,
    {
        let x = self.methods.read().unwrap();
        x.iter()
            /* cSpell:disable-next-line */
            .rposition(|m| unsafe { m.as_ref().name().as_str().eq(name) })
            .map(|x| x as u32)
    }

    pub fn get_method_by_ref(&self, r: &MethodRef) -> Option<NonNull<Method<T>>> {
        match r {
            MethodRef::Index(i) => self.get_method(*i).map(|x| *x),
            MethodRef::Specific { index, types } => self
                .get_method(*index)
                .map(|x| unsafe { x.as_ref().instantiate(types) }),
        }
    }

    pub fn get_static_constructor(&self) -> MappedRwLockReadGuard<'_, NonNull<Method<T>>>
    where
        T: GetStaticConstructorId,
    {
        self.get_method(self.ty_ref().__get_static_constructor_id())
            .unwrap()
    }
}

impl<T> MethodTable<T>
where
    T: GetAssemblyRef + GetMethodTableRef + GetParent + 'static,
{
    pub fn wrap_as_method_generator<F: FnOnce(NonNull<Self>) -> Vec<Box<Method<T>>>>(
        f: F,
    ) -> impl FnOnce(NonNull<T>) -> NonNull<Self> {
        move |ty| Self::new(ty, f).as_non_null_ptr()
    }
    /// The NonNull passed to method_generator is always valid to be cast to &Self
    pub fn new<F: FnOnce(NonNull<Self>) -> Vec<Box<Method<T>>>>(
        ty: NonNull<T>,
        method_generator: F,
    ) -> Unique<Self> {
        let this = Box::new(Self {
            ty,
            methods: RwLock::new(Vec::new()),
            __override_methods: Vec::new(),
            cached_layout: Cell::new(None),
            cached_static_layout: Cell::new(None),
        });

        let mut this = Box::into_non_null(this);

        let mut methods = Vec::new();
        if let Some(parent) = unsafe { ty.as_ref() }
            .__get_parent()
            .map(|x| unsafe { x.as_ref().__get_method_table_ref() })
        {
            let parent_methods = parent.methods.read().unwrap();
            // Stop on the first static method
            methods.extend(parent_methods.iter().map_while(|x| {
                if unsafe { x.as_ref().attr().is_static() } {
                    None
                } else {
                    Some(*x)
                }
            }));
        }
        let this_methods = method_generator(this);

        for this_m in this_methods.into_iter() {
            if let Some(o) = this_m.attr().overrides().as_ref().copied() {
                methods[o as usize] = Box::into_non_null(this_m);
                unsafe {
                    this.as_mut().__override_methods.push(o as usize);
                }
            } else {
                methods.push(Box::into_non_null(this_m));
            }
        }
        unsafe {
            this.as_mut().methods = RwLock::new(methods);
        }

        Unique::from_non_null(this)
    }
}

impl<T> MethodTable<T>
where
    T: GetAssemblyRef + GetGeneric + GetMethodTableRef,
{
    pub fn get_core_type_id(&self) -> Option<CoreTypeId> {
        let assem = self.ty_ref().__get_assembly_ref();
        if !assem.is_core {
            return None;
        }

        let types = assem.types.read().unwrap();
        let id = if let Some(g) = self.ty_ref().__get_generic() {
            return unsafe { g.as_ref().__get_method_table_ref().get_core_type_id() };
        } else {
            types.iter().position(|x| match x {
                super::type_handle::NonGenericTypeHandle::Class(ty) => unsafe {
                    std::ptr::addr_eq(self, ty.as_ref().method_table_ref())
                },
                super::type_handle::NonGenericTypeHandle::Struct(ty) => unsafe {
                    std::ptr::addr_eq(self, ty.as_ref().method_table_ref())
                },
            })? as u32
        };

        CoreTypeId::try_from(id).ok()
    }
}

impl<T> MethodTable<T>
where
    T: GetFields<Field = super::field::Field>
        + GetTypeVars
        + GetAssemblyRef
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    pub fn core_mem_layout(&self) -> Option<Layout> {
        match self.get_core_type_id() {
            Some(x) => x.mem_layout(),
            None => None,
        }
    }
    pub fn mem_layout(&self, options: GetLayoutOptions) -> Layout {
        if options.prefer_cached
            && let Some(cached_layout) = self.cached_layout.get()
        {
            return cached_layout;
        }

        let layout = if let Some(core_layout) = self.core_mem_layout() {
            core_layout
        } else {
            self.calc_layout()
        };

        if !options.discard_calculated_layout {
            self.cached_layout.set(Some(layout));
        }

        layout
    }

    pub fn static_layout(&self, options: GetLayoutOptions) -> Layout {
        if options.prefer_cached
            && let Some(cached_static_layout) = self.cached_static_layout.get()
        {
            return cached_static_layout;
        }

        let layout = self.calc_static_layout();

        if !options.discard_calculated_layout {
            self.cached_static_layout.set(Some(layout));
        }

        layout
    }

    fn _common_calc_layout<F: Fn(&Field) -> bool>(&self, check: &F) -> Layout {
        let mut total = self
            .ty_ref()
            .__get_parent()
            .map(|x| unsafe {
                x.as_ref()
                    .__get_method_table_ref()
                    ._common_calc_layout(check)
            })
            .unwrap_or_else(Layout::new::<()>);

        // Little hack for casting immutable to mutable
        for f in unsafe { NonNull::from_ref(self).as_mut() }
            .ty_mut()
            .__get_fields_mut()
        {
            if !check(f) {
                continue;
            }

            (total, _) = total
                .extend(f.layout_with_type(self.ty_ref(), GetLayoutOptions::default()))
                .unwrap();
        }

        total
    }

    fn calc_layout(&self) -> Layout {
        self._common_calc_layout(&|x| !x.attr().is_static())
    }
    fn calc_static_layout(&self) -> Layout {
        self._common_calc_layout(&|x| x.attr().is_static())
    }
}

#[derive(Copy, derive_more::Debug)]
#[derive_const(Clone)]
pub struct FieldMemInfo {
    pub offset: usize,
    pub layout: Layout,
    #[debug("{}", ty.name())]
    pub ty: NonGenericTypeHandle,
}

impl<T> MethodTable<T>
where
    T: GetFields<Field = super::field::Field>
        + GetTypeVars
        + GetAssemblyRef
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    unsafe fn _common_field_mem_info_unchecked(
        &self,
        i: u32,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> FieldMemInfo {
        let mut total_layout = self
            .ty_ref()
            .__get_parent()
            .map(|x| unsafe {
                x.as_ref()
                    .__get_method_table_ref()
                    ._common_calc_layout(&check)
            })
            .unwrap_or_else(Layout::new::<()>);
        let mut offset = 0;
        // Little hack for casting immutable to mutable
        let fields = self.ty_ref().__get_fields();

        if offset_options.prefer_cached
            && layout_options.prefer_cached
            && let field = unsafe { fields.get_unchecked(i as usize) }
            && let Some(offset) = get_cached_offset(field)
            && let Some(layout) = field.cached_layout.get()
        {
            return FieldMemInfo {
                offset,
                layout,
                ty: field.get_type_with_type(self.ty_ref()),
            };
        }
        let fields_mut = unsafe { fields.get_unchecked(..=(i as usize)) };

        for f in fields_mut {
            if !check(f) {
                continue;
            }
            (total_layout, offset) = total_layout
                .extend(f.layout_with_type(self.ty_ref(), layout_options))
                .unwrap();
        }

        let field = unsafe { fields.get_unchecked(i as usize) };

        if !offset_options.discard_calculated_offset {
            set_cached_offset(field, offset);
        }

        let ty = field.get_type_with_type(self.ty_ref());

        let layout = ty.val_layout();
        if !layout_options.discard_calculated_layout {
            field.cached_layout.set(Some(layout));
        }

        FieldMemInfo { offset, layout, ty }
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn _common_field_offset_unchecked(
        &self,
        i: u32,
        options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> usize {
        self._common_field_mem_info_unchecked(
            i,
            Default::default(),
            options,
            check,
            get_cached_offset,
            set_cached_offset,
        )
        .offset
    }

    fn _common_field_mem_info(
        &self,
        i: u32,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> Option<FieldMemInfo> {
        if (i as usize) >= self.ty_ref().__get_fields().len() {
            None
        } else {
            unsafe {
                Some(self._common_field_mem_info_unchecked(
                    i,
                    layout_options,
                    offset_options,
                    check,
                    get_cached_offset,
                    set_cached_offset,
                ))
            }
        }
    }

    fn _common_all_fields_mem_info(
        &self,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> Vec<FieldMemInfo> {
        let mut total_layout = self
            .ty_ref()
            .__get_parent()
            .map(|x| unsafe {
                x.as_ref()
                    .__get_method_table_ref()
                    ._common_calc_layout(&check)
            })
            .unwrap_or_else(Layout::new::<()>);
        let mut offset = 0;

        // Little hack for casting immutable to mutable
        let fields = self.ty_ref().__get_fields();
        let mut result = Vec::new();

        for field in fields {
            if !check(field) {
                continue;
            }
            if offset_options.prefer_cached
                && layout_options.prefer_cached
                && let Some(_offset) = get_cached_offset(field)
                && let Some(layout) = field.cached_layout.get()
            {
                result.push(FieldMemInfo {
                    offset: _offset,
                    layout,
                    ty: field.get_type_with_type(self.ty_ref()),
                });
                (total_layout, offset) = total_layout.extend(layout).unwrap();
                continue;
            }
            let ty = field.get_type_with_type(self.ty_ref());
            let field_layout = ty.val_layout();
            if !layout_options.discard_calculated_layout {
                field.cached_layout.set(Some(field_layout));
            }
            result.push(FieldMemInfo {
                offset,
                layout: field_layout,
                ty,
            });
            (total_layout, offset) = total_layout.extend(field_layout).unwrap();
            if !offset_options.discard_calculated_offset {
                set_cached_offset(field, offset);
            }
        }

        result
    }

    fn _common_field_offset(
        &self,
        i: u32,
        options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> Option<usize> {
        if (i as usize) >= self.ty_ref().__get_fields().len() {
            None
        } else {
            unsafe {
                Some(self._common_field_offset_unchecked(
                    i,
                    options,
                    check,
                    get_cached_offset,
                    set_cached_offset,
                ))
            }
        }
    }

    fn _common_all_fields_offset(
        &self,
        options: GetFieldOffsetOptions,
        check: fn(&Field) -> bool,
        get_cached_offset: fn(&Field) -> Option<usize>,
        set_cached_offset: fn(&Field, usize),
    ) -> Vec<usize> {
        self._common_all_fields_mem_info(
            Default::default(),
            options,
            check,
            get_cached_offset,
            set_cached_offset,
        )
        .into_iter()
        .map(|x| x.offset)
        .collect()
    }

    pub fn field_mem_info(
        &self,
        i: u32,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
    ) -> Option<FieldMemInfo> {
        self._common_field_mem_info(
            i,
            layout_options,
            offset_options,
            field_helper::check,
            field_helper::get_cached_offset,
            field_helper::set_cached_offset,
        )
    }

    pub fn all_fields_mem_info(
        &self,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
    ) -> Vec<FieldMemInfo> {
        self._common_all_fields_mem_info(
            layout_options,
            offset_options,
            field_helper::check,
            field_helper::get_cached_offset,
            field_helper::set_cached_offset,
        )
    }
    pub fn static_field_mem_info(
        &self,
        i: u32,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
    ) -> Option<FieldMemInfo> {
        self._common_field_mem_info(
            i,
            layout_options,
            offset_options,
            field_helper::check_static,
            field_helper::get_cached_offset_static,
            field_helper::set_cached_offset_static,
        )
    }
    pub fn all_static_fields_mem_info(
        &self,
        layout_options: GetLayoutOptions,
        offset_options: GetFieldOffsetOptions,
    ) -> Vec<FieldMemInfo> {
        self._common_all_fields_mem_info(
            layout_options,
            offset_options,
            field_helper::check_static,
            field_helper::get_cached_offset_static,
            field_helper::set_cached_offset_static,
        )
    }
    pub fn field_offset(&self, i: u32, options: GetFieldOffsetOptions) -> Option<usize> {
        self._common_field_offset(
            i,
            options,
            field_helper::check,
            field_helper::get_cached_offset,
            field_helper::set_cached_offset,
        )
    }
    pub fn static_field_offset(&self, i: u32, options: GetFieldOffsetOptions) -> Option<usize> {
        self._common_field_offset(
            i,
            options,
            field_helper::check_static,
            field_helper::get_cached_offset_static,
            field_helper::set_cached_offset_static,
        )
    }

    pub fn all_fields_offset(&self, options: GetFieldOffsetOptions) -> Vec<usize> {
        self._common_all_fields_offset(
            options,
            field_helper::check,
            field_helper::get_cached_offset,
            field_helper::set_cached_offset,
        )
    }
    pub fn all_static_fields_offset(&self, options: GetFieldOffsetOptions) -> Vec<usize> {
        self._common_all_fields_offset(
            options,
            field_helper::check_static,
            field_helper::get_cached_offset_static,
            field_helper::set_cached_offset_static,
        )
    }
}

mod field_helper {
    use crate::type_system::field::Field;

    #[inline(always)]
    pub const fn get_cached_offset(f: &Field) -> Option<usize> {
        f.cached_offset.get()
    }
    #[inline(always)]
    pub const fn set_cached_offset(f: &Field, offset: usize) {
        f.cached_offset.set(Some(offset));
    }
    #[inline(always)]
    pub const fn get_cached_offset_static(f: &Field) -> Option<usize> {
        f.cached_static_offset.get()
    }
    #[inline(always)]
    pub const fn set_cached_offset_static(f: &Field, offset: usize) {
        f.cached_static_offset.set(Some(offset));
    }
    #[inline(always)]
    pub fn check(f: &Field) -> bool {
        !f.attr().is_static()
    }
    #[inline(always)]
    pub fn check_static(f: &Field) -> bool {
        f.attr().is_static()
    }
}

trait DropSpec {
    fn __drop(&mut self);
}

impl<T> DropSpec for MethodTable<T> {
    default fn __drop(&mut self) {
        let mut methods = self.methods.write().unwrap();
        for m in methods
            .iter_mut()
            .enumerate()
            .filter(|x| !self.__override_methods.contains(&x.0))
            .map(|x| x.1)
        {
            unsafe {
                drop(Box::from_non_null(*m));
            }
            *m = NonNull::dangling();
        }
        drop(methods);
        common_drop_method_table(self);
    }
}

fn common_drop_method_table<T>(#[allow(unused)] this: &mut MethodTable<T>) {}

impl<T: GetParent + GetMethodTableRef> DropSpec for MethodTable<T> {
    fn __drop(&mut self) {
        let parent_mt_len = self
            .ty_ref()
            .__get_parent()
            .map(|p| {
                unsafe { p.as_ref().__get_method_table_ref() }
                    .methods
                    .read()
                    .unwrap()
                    .len()
            })
            .unwrap_or(0);

        let mut methods = self.methods.replace(Vec::new()).unwrap();
        methods.shrink_to_fit();
        let (parent_methods, this_methods) = methods.split_at(parent_mt_len);
        this_methods.iter().for_each(|x| unsafe {
            drop(Box::from_non_null(*x));
        });
        for ov in &self.__override_methods {
            unsafe {
                drop(Box::from_non_null(parent_methods[*ov]));
            }
        }
        common_drop_method_table(self);
    }
}

impl<T> Drop for MethodTable<T> {
    fn drop(&mut self) {
        DropSpec::__drop(self);
    }
}
