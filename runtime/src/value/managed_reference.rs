use std::{
    alloc::{Layout, LayoutError},
    mem::offset_of,
    ptr::NonNull,
};

use crate::{
    stdlib::System_Object_MethodId,
    type_system::{
        class::Class,
        field::Field,
        get_traits::{
            GetAssemblyRef, GetFields, GetGeneric, GetMethodTableRef, GetParent, GetTypeVars,
        },
        method_table::MethodTable,
    },
    value::object_header::ObjectHeader,
    virtual_machine::cpu::CPU,
};

trait IAccessor<T>: Sized {
    fn is_valid(r: &ManagedReference<T>) -> bool;
}

pub use for_array::ArrayAccessor;
pub use for_string::StringAccessor;

#[repr(transparent)]
pub struct ManagedReference<T> {
    pub(crate) data: Option<NonNull<ManagedReferenceInner<T>>>,
}

impl<T> const Clone for ManagedReference<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ManagedReference<T> {}

impl<T> ManagedReference<T> {
    pub const fn null() -> Self {
        Self { data: None }
    }

    pub const fn data(&self) -> Option<NonNull<()>> {
        const fn op<T>(x: NonNull<ManagedReferenceInner<T>>) -> NonNull<()> {
            unsafe {
                x.cast()
                    .byte_add(std::mem::offset_of!(ManagedReferenceInner<T>, data))
            }
        }
        self.data.map(op::<T>)
    }

    pub const fn method_table(&self) -> Option<&NonNull<MethodTable<T>>> {
        const fn op<'a, T>(x: NonNull<ManagedReferenceInner<T>>) -> &'a NonNull<MethodTable<T>> {
            unsafe {
                x.byte_add(offset_of!(ManagedReferenceInner<T>, mt))
                    .cast::<NonNull<MethodTable<T>>>()
                    .as_ref()
            }
        }
        self.data.map(op::<T>)
    }

    pub const fn method_table_ref(&self) -> Option<&MethodTable<T>> {
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

    pub const fn as_raw_ptr(&self) -> *mut ManagedReferenceInner<T> {
        self.data
            .map(NonNull::as_ptr)
            .unwrap_or_else(std::ptr::null_mut)
    }

    pub const fn cast<U>(&self) -> ManagedReference<U> {
        ManagedReference {
            data: self.data.map(NonNull::cast),
        }
    }

    pub const fn erase(&self) -> ManagedReference<()> {
        self.cast()
    }

    pub const fn is_null(&self) -> bool {
        self.data.is_none()
    }
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

    /// # Safety
    /// TAccessor's layout must be same as Self
    #[inline(always)]
    pub unsafe fn access_unchecked<TAccessor: IAccessor<T>>(&self) -> &TAccessor {
        unsafe { std::mem::transmute(self) }
    }
    /// # Safety
    /// TAccessor's layout must be same as Self
    #[inline(always)]
    pub unsafe fn access_unchecked_mut<TAccessor: IAccessor<T>>(&mut self) -> &mut TAccessor {
        unsafe { std::mem::transmute(self) }
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
    /// None if data is None or [`Self::calc_full_layout`] fails
    pub fn get_full_layout(&self) -> Option<Layout> {
        let data = unsafe { self.data?.as_ref() };
        let mem_layout = if data.header.is_static() {
            unsafe { data.mt.as_ref() }.mem_layout(Default::default())
        } else {
            unsafe { data.mt.as_ref() }.static_layout(Default::default())
        };
        Self::calc_full_layout(mem_layout).ok()
    }
}

mod for_array;
mod for_class;
mod for_string;
mod for_struct;

trait DestroySpec {
    fn get_mem_layout(&self) -> Option<Layout>;
    fn destroy_spec(&mut self, cpu: &CPU);
}

impl DestroySpec for ManagedReference<Class> {
    fn get_mem_layout(&self) -> Option<Layout> {
        if let Some(accessor) = self.access::<ArrayAccessor>() {
            return crate::memory::arrayed_layout(accessor.element_layout()?, accessor.len()?);
        }
        let data = unsafe { self.data?.as_ref() };
        Some(if data.header.is_static() {
            unsafe { data.mt.as_ref() }.mem_layout(Default::default())
        } else {
            unsafe { data.mt.as_ref() }.static_layout(Default::default())
        })
    }
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

            let destructor = *self
                .method_table_ref_unchecked()
                .get_method(System_Object_MethodId::Destructor as _)
                .unwrap();

            destructor.as_ref().typed_res_call::<()>(
                cpu,
                Some(NonNull::from_mut(self).cast()),
                &[],
            );
        }
    }
}

#[allow(private_bounds)]
impl<T> ManagedReference<T>
where
    Self: DestroySpec,
{
    pub fn destroy(&mut self, cpu: &CPU) {
        self.destroy_spec(cpu);
    }
}

pub struct ManagedReferenceInner<T> {
    pub(crate) header: ObjectHeader,
    pub(crate) mt: NonNull<MethodTable<T>>,
    pub(crate) data: [u8; 0],
}
