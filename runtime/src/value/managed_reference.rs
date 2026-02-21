use std::{
    alloc::{Layout, LayoutError},
    fmt::Debug,
    mem::offset_of,
    ptr::NonNull,
};

use crate::{
    stdlib::System_Object_MethodId,
    type_system::{
        class::Class,
        field::Field,
        get_traits::{
            GetAssemblyRef, GetFields, GetGeneric, GetMethodTableRef, GetNonGenericTypeHandleKind,
            GetParent, GetTypeVars,
        },
        method_table::MethodTable,
        r#struct::Struct,
    },
    value::object_header::ObjectHeader,
    virtual_machine::cpu::CPU,
};

const trait IAccessor<T>: Sized {
    fn is_valid(r: &ManagedReference<T>) -> bool;
}

pub use for_array::ArrayAccessor;
pub use for_field::FieldAccessor;
pub use for_large_string::LargeStringAccessor;
pub use for_string::StringAccessor;

#[repr(transparent)]
pub struct ManagedReference<T> {
    pub(crate) data: Option<NonNull<ManagedReferenceInner<T>>>,
}

#[test]
#[ignore = "Compile time check only"]
fn __test_layout() {
    fn __f()
    where
        global::assertions::LayoutEq<ManagedReference<Class>, NonNull<u8>>:
            global::assertions::SuccessAssert,
    {
    }
    __f();
}

impl<T> Debug for ManagedReference<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as std::fmt::Pointer>::fmt(self, f)
    }
}

impl<T> std::fmt::Display for ManagedReference<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as std::fmt::Pointer>::fmt(self, f)
    }
}

impl<T> std::fmt::Pointer for ManagedReference<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as std::fmt::Pointer>::fmt(&self.as_raw_ptr(), f)
    }
}

impl<T> const Clone for ManagedReference<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ManagedReference<T> {}

impl<T> ManagedReference<T> {
    #[inline(always)]
    pub const fn null() -> Self {
        Self { data: None }
    }

    /// Pointer to data (differs from [`ManagedReference`]'s data field)
    #[inline]
    pub const fn data(&self) -> Option<NonNull<ManagedReferenceData>> {
        #[inline]
        const fn op<T>(x: NonNull<ManagedReferenceInner<T>>) -> NonNull<ManagedReferenceData> {
            unsafe {
                x.cast()
                    .byte_add(std::mem::offset_of!(ManagedReferenceInner<T>, data))
            }
        }
        self.data.map(op::<T>)
    }

    /// Pointer to data (differs from [`ManagedReference`]'s data field)
    #[inline]
    pub const fn data_ptr(&self) -> *mut ManagedReferenceData {
        self.data()
            .map(NonNull::as_ptr)
            .unwrap_or_else(std::ptr::null_mut)
    }

    #[inline]
    pub const fn method_table(&self) -> Option<&NonNull<MethodTable<T>>> {
        #[inline]
        const fn op<'a, T>(x: NonNull<ManagedReferenceInner<T>>) -> &'a NonNull<MethodTable<T>> {
            unsafe {
                x.byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                    .cast::<NonNull<MethodTable<T>>>()
                    .as_ref()
            }
        }
        self.data.map(op::<T>)
    }

    #[inline]
    pub const fn method_table_ref(&self) -> Option<&MethodTable<T>> {
        #[inline]
        const fn op<'a, T>(x: NonNull<ManagedReferenceInner<T>>) -> &'a MethodTable<T> {
            unsafe {
                x.byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                    .cast::<NonNull<MethodTable<T>>>()
                    .read()
                    .as_ref()
            }
        }
        self.data.map(op::<T>)
    }

    #[inline]
    #[contracts::requires(!self.is_null())]
    pub const unsafe fn method_table_unchecked(&self) -> *mut NonNull<MethodTable<T>> {
        unsafe {
            self.as_raw_ptr()
                .byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                .cast::<NonNull<MethodTable<T>>>()
        }
    }

    #[contracts::requires(!self.is_null())]
    pub const unsafe fn method_table_ref_unchecked(&self) -> &MethodTable<T> {
        unsafe {
            self.as_raw_ptr()
                .byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                .cast::<NonNull<MethodTable<T>>>()
                .read()
                .as_ref()
        }
    }

    #[inline]
    pub const fn header_ptr(&self) -> *mut ObjectHeader {
        unsafe {
            self.as_raw_ptr()
                .byte_add(offset_of!(ManagedReferenceInner<T>, header))
                .cast::<ObjectHeader>()
        }
    }

    #[inline]
    pub const fn header(&self) -> Option<&ObjectHeader> {
        unsafe { self.header_ptr().as_ref() }
    }
    #[inline]
    pub const fn header_mut(&mut self) -> Option<&mut ObjectHeader> {
        unsafe { self.header_ptr().as_mut() }
    }
    #[inline]
    #[contracts::requires(!self.is_null())]
    pub const unsafe fn header_unchecked(&self) -> &ObjectHeader {
        unsafe { self.header_ptr().as_ref_unchecked() }
    }
    #[inline]
    #[contracts::requires(!self.is_null())]
    pub const unsafe fn header_unchecked_mut(&mut self) -> &mut ObjectHeader {
        unsafe { self.header_ptr().as_mut_unchecked() }
    }

    #[inline]
    pub const fn as_raw_ptr(&self) -> *mut ManagedReferenceInner<T> {
        self.data
            .map(NonNull::as_ptr)
            .unwrap_or_else(std::ptr::null_mut)
    }

    #[inline]
    pub const fn cast<U>(&self) -> ManagedReference<U> {
        ManagedReference {
            data: self.data.map(NonNull::cast),
        }
    }

    #[inline]
    pub const fn erase(&self) -> ManagedReference<()> {
        self.cast()
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.data.is_none()
    }
}

const fn const_accessor_valid_check<T, TAccessor: const IAccessor<T>>() {
    assert!(TAccessor::is_valid(&ManagedReference::null()));
}

#[allow(private_bounds)]
impl<T> ManagedReference<T> {
    pub fn is_accessor_valid<TAccessor: IAccessor<T>>(&self) -> bool {
        TAccessor::is_valid(self)
    }

    pub fn access<TAccessor: IAccessor<T>>(&self) -> Option<&TAccessor> {
        self.is_accessor_valid::<TAccessor>()
            .then_some(unsafe { self.access_unchecked() })
    }

    pub fn access_mut<TAccessor: IAccessor<T>>(&mut self) -> Option<&mut TAccessor> {
        self.is_accessor_valid::<TAccessor>()
            .then_some(unsafe { self.access_unchecked_mut() })
    }

    #[inline(always)]
    pub const fn const_access<TAccessor: const IAccessor<T>>(&self) -> &TAccessor {
        const { const_accessor_valid_check::<T, TAccessor>() };

        unsafe { self.access_unchecked() }
    }

    #[inline(always)]
    pub const fn const_access_mut<TAccessor: const IAccessor<T>>(&mut self) -> &mut TAccessor {
        const { const_accessor_valid_check::<T, TAccessor>() };

        unsafe { self.access_unchecked_mut() }
    }

    /// # Safety
    /// - `TAccessor`'s layout must be same as Self
    /// - `TAccessor` must be valid(i.e. [`IAccessor::is_valid`] returns true)
    #[inline(always)]
    pub const unsafe fn access_unchecked<TAccessor: IAccessor<T>>(&self) -> &TAccessor {
        unsafe { &*(self as *const Self as *const TAccessor) }
    }
    /// # Safety
    /// - `TAccessor`'s layout must be same as Self
    /// - `TAccessor` must be valid(i.e. [`IAccessor::is_valid`] returns true)
    #[inline(always)]
    pub const unsafe fn access_unchecked_mut<TAccessor: IAccessor<T>>(&mut self) -> &mut TAccessor {
        unsafe { &mut *(self as *mut Self as *mut TAccessor) }
    }
}

impl<T> ManagedReference<T>
where
    T: GetAssemblyRef
        + GetFields<Field = Field>
        + GetTypeVars
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    #[inline]
    pub fn base_common_alloc<Recorder: FnOnce(Self)>(
        mt: NonNull<MethodTable<T>>,
        is_static: bool,
        recorder: Recorder,
    ) -> Self {
        let mt_ref = unsafe { mt.as_ref() };
        let layout = if is_static {
            mt_ref.static_layout(Default::default())
        } else {
            mt_ref.mem_layout(Default::default())
        };
        let full_layout = Self::calc_full_layout(layout).unwrap();
        let ptr = std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, full_layout).unwrap();
        unsafe {
            ptr.cast::<ObjectHeader>()
                .write(ObjectHeader::new(is_static));
            ptr.byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                .cast::<NonNull<MethodTable<T>>>()
                .write(mt);
        }

        let ptr = ptr.cast();
        let this = Self { data: Some(ptr) };
        recorder(this);

        this
    }

    pub fn common_alloc(cpu: &CPU, mt: NonNull<MethodTable<T>>, is_static: bool) -> Self
    where
        T: GetNonGenericTypeHandleKind,
    {
        Self::base_common_alloc(
            mt,
            is_static,
            cpu.gen_mem_recorder(T::__get_non_generic_type_handle_kind(unsafe {
                mt.as_ref().ty_ref()
            })),
        )
    }
}

impl<T> ManagedReference<T> {
    const fn calc_full_layout(mem_layout: Layout) -> Result<Layout, LayoutError> {
        let mut full_layout = Layout::new::<ObjectHeader>();
        (full_layout, _) = full_layout.extend(Layout::new::<NonNull<MethodTable<T>>>())?;
        (full_layout, _) = full_layout.extend(mem_layout)?;

        Ok(full_layout)
    }
}

impl<T> ManagedReference<T>
where
    T: GetAssemblyRef
        + GetFields<Field = Field>
        + GetTypeVars
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    fn get_mem_layout(&self) -> Option<Layout> {
        let data = unsafe { self.data?.as_ref() };
        if data.header.is_static() {
            Some(unsafe { data.mt.as_ref() }.static_layout(Default::default()))
        } else if let Some(accessor) = self.access::<ArrayAccessor>() {
            let element_layouts =
                crate::memory::arrayed_layout(accessor.element_layout()?, accessor.len()?)?;
            Layout::new::<usize>()
                .extend(element_layouts)
                .ok()
                .map(|(x, _)| x)
        } else if let Some(accessor) = self.access::<StringAccessor>() {
            let s = accessor.get_str()?;
            Some(Layout::for_value(s))
        } else if let Some(accessor) = self.access::<LargeStringAccessor>() {
            let s = accessor.as_str()?;
            Layout::new::<usize>()
                .extend(Layout::for_value(s))
                .ok()
                .map(|(x, _)| x)
        } else {
            Some(unsafe { data.mt.as_ref() }.mem_layout(Default::default()))
        }
    }
    /// None if data is None or [`Self::calc_full_layout`] fails
    pub fn get_full_layout(&self) -> Option<Layout> {
        let mem_layout = self.get_mem_layout()?;
        Self::calc_full_layout(mem_layout).ok()
    }
}

mod for_array;
mod for_class;
mod for_field;
mod for_large_string;
mod for_string;
mod for_struct;

trait DestroySpec {
    fn destroy_spec(&mut self, cpu: &CPU);
}

trait CallDestructorSpec: Sized {
    fn call_destructor_spec(r: &ManagedReference<Self>, cpu: &CPU);
}

trait GcMarkSpec {
    fn set_marker_spec(&mut self, val: bool);
}

impl CallDestructorSpec for Class {
    fn call_destructor_spec(r: &ManagedReference<Self>, cpu: &CPU) {
        unsafe {
            let destructor = *r
                .method_table_ref_unchecked()
                .get_method(System_Object_MethodId::Destructor as _)
                .unwrap();
            destructor
                .as_ref()
                .typed_res_call::<()>(cpu, Some(NonNull::from_ref(r).cast()), &[]);
        }
    }
}

impl CallDestructorSpec for Struct {
    #[inline(always)]
    fn call_destructor_spec(_: &ManagedReference<Self>, _: &CPU) {}
}

impl DestroySpec for ManagedReference<Class> {
    fn destroy_spec(&mut self, cpu: &CPU) {
        if self.is_null() {
            return;
        }
        let full_layout = Self::calc_full_layout(self.get_mem_layout().unwrap()).unwrap();
        let Some(data) = self.data else {
            return;
        };
        unsafe {
            data.byte_add(offset_of!(ManagedReferenceInner<Class>, header))
                .cast::<ObjectHeader>()
                .as_mut()
                .sync()
                .destroy();

            Class::call_destructor_spec(self, cpu);

            std::alloc::Allocator::deallocate(&std::alloc::Global, data.cast(), full_layout);
        }

        self.data = None;
    }
}

impl DestroySpec for ManagedReference<Struct> {
    fn destroy_spec(&mut self, _: &CPU) {
        if self.is_null() {
            return;
        }
        let full_layout = Self::calc_full_layout(self.get_mem_layout().unwrap()).unwrap();
        let Some(data) = self.data else {
            return;
        };
        unsafe {
            data.byte_add(offset_of!(ManagedReferenceInner<Class>, header))
                .cast::<ObjectHeader>()
                .as_mut()
                .sync()
                .destroy();

            std::alloc::Allocator::deallocate(&std::alloc::Global, data.cast(), full_layout);
        }

        self.data = None;
    }
}

impl<T> GcMarkSpec for ManagedReference<T> {
    default fn set_marker_spec(&mut self, val: bool) {
        if let Some(header) = self.header_mut() {
            header.set_is_marked(val);
        }
    }
}

impl GcMarkSpec for ManagedReference<Class> {
    fn set_marker_spec(&mut self, val: bool) {
        if let Some(header) = self.header_mut() {
            header.set_is_marked(val);
        }
    }
}

#[allow(private_bounds)]
impl<T> ManagedReference<T>
where
    Self: DestroySpec,
{
    /// This method should never be called except GC
    pub fn destroy(&mut self, cpu: &CPU) {
        self.destroy_spec(cpu);
    }
}

#[allow(private_bounds)]
impl<T> ManagedReference<T>
where
    T: CallDestructorSpec,
{
    /// This method should never be called except GC
    pub fn call_destructor(&mut self, cpu: &CPU) {
        T::call_destructor_spec(self, cpu);
    }
}

impl<T> ManagedReference<T> {
    pub fn set_marker(&mut self, val: bool) {
        self.set_marker_spec(val);
    }
}

#[repr(C)]
pub struct ManagedReferenceInner<T> {
    pub(crate) header: ObjectHeader,
    pub(crate) mt: NonNull<MethodTable<T>>,
    pub(crate) data: ManagedReferenceData,
}

pub enum ManagedReferenceData {}
