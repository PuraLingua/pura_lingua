use std::{alloc::Layout, ptr::NonNull};

use crate::{
    memory::GetFieldOffsetOptions,
    type_system::{
        class::Class,
        field::Field,
        get_traits::{
            GetAssemblyRef, GetFields, GetGeneric, GetMethodTableRef, GetParent, GetTypeVars,
        },
        method_table::FieldMemInfo,
        r#struct::Struct,
        type_handle::NonGenericTypeHandle,
    },
};

use super::{IAccessor, ManagedReference};

#[repr(transparent)]
pub struct FieldAccessor<T>(ManagedReference<T>);

impl<T> const IAccessor<T> for FieldAccessor<T> {
    #[inline(always)]
    default fn is_valid(_: &ManagedReference<T>) -> bool {
        false
    }
}

impl<T> const IAccessor<T> for FieldAccessor<T>
where
    T: GetFields<Field = Field>
        + GetTypeVars
        + GetAssemblyRef
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    fn is_valid(_: &ManagedReference<T>) -> bool {
        true
    }
}

impl<T> FieldAccessor<T>
where
    T: GetFields<Field = Field>
        + GetTypeVars
        + GetAssemblyRef
        + GetParent
        + GetMethodTableRef
        + GetGeneric,
{
    pub fn all_fields(
        &self,
        options: GetFieldOffsetOptions,
    ) -> impl Iterator<Item = (NonNull<u8>, Layout, NonGenericTypeHandle)> {
        let map_info = |info: FieldMemInfo| {
            let f_ptr = unsafe { self.0.data_ptr().byte_add(info.offset) };
            (
                unsafe { NonNull::new_unchecked(f_ptr.cast()) },
                info.layout,
                info.ty,
            )
        };
        let Some(is_static) = self.0.header().map(|x| x.is_static()) else {
            return Vec::new().into_iter().map(map_info);
        };
        let Some(mt) = self.0.method_table_ref() else {
            return Vec::new().into_iter().map(map_info);
        };

        if is_static {
            mt.all_static_fields_mem_info(Default::default(), options)
                .into_iter()
                .map(map_info)
        } else {
            mt.all_fields_mem_info(Default::default(), options)
                .into_iter()
                .map(map_info)
        }
    }
}

impl FieldAccessor<Class> {
    pub fn field(&self, i: u32, options: GetFieldOffsetOptions) -> Option<(NonNull<u8>, Layout)> {
        let mt = self.0.method_table_ref()?;
        let FieldMemInfo { offset, layout, .. } =
            mt.field_mem_info(i, Default::default(), options)?;

        Some((unsafe { self.0.data()?.byte_add(offset).cast() }, layout))
    }

    pub fn typed_field<T>(&self, i: u32, options: GetFieldOffsetOptions) -> Option<&T> {
        let (f_ptr, layout) = self.field(i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_ref() })
    }

    pub fn typed_field_mut<T>(&mut self, i: u32, options: GetFieldOffsetOptions) -> Option<&mut T> {
        let (f_ptr, layout) = self.field(i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_mut() })
    }

    pub fn read_typed_field<T>(&self, i: u32, options: GetFieldOffsetOptions) -> Option<T> {
        let (f_ptr, layout) = self.field(i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        unsafe { Some(f_ptr.cast::<T>().read()) }
    }

    /// Return true if success
    #[must_use]
    pub fn write_typed_field<T: Copy>(
        &mut self,
        i: u32,
        options: GetFieldOffsetOptions,
        val: T,
    ) -> bool {
        let Some((f_ptr, layout)) = self.field(i, options) else {
            return false;
        };
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        unsafe {
            f_ptr.cast::<T>().write(val);
        }
        true
    }
}

impl FieldAccessor<Struct> {
    pub fn field(&self, i: u32, options: GetFieldOffsetOptions) -> Option<(NonNull<u8>, Layout)> {
        let is_static = self.0.header()?.is_static();
        let mt = self.0.method_table_ref()?;
        let offset = if is_static {
            mt.static_field_offset(i, options)?
        } else {
            mt.field_offset(i, options)?
        };

        let f_ptr = unsafe { self.0.data()?.byte_add(offset).cast() };
        let f_layout =
            mt.ty_ref().fields()[i as usize].layout_with_type(mt.ty_ref(), Default::default());

        Some((f_ptr, f_layout))
    }

    pub fn typed_field<T>(&self, i: u32, options: GetFieldOffsetOptions) -> Option<&T> {
        let (f_ptr, layout) = self.field(i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_ref() })
    }

    pub fn typed_field_mut<T>(&mut self, i: u32, options: GetFieldOffsetOptions) -> Option<&mut T> {
        let (f_ptr, layout) = self.field(i, options)?;
        debug_assert!(Layout::new::<T>().size() <= layout.size());
        Some(unsafe { f_ptr.cast::<T>().as_mut() })
    }
}
