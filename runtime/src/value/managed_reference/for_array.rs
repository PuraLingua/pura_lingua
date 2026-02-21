use std::{alloc::Layout, intrinsics::const_eval_select, mem::offset_of, ptr::NonNull};

use crate::{
    stdlib::CoreTypeId,
    type_system::{
        class::Class,
        get_traits::GetValLayout,
        method_table::MethodTable,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandleKind, TypeHandle},
    },
    value::{managed_reference::ManagedReferenceInner, object_header::ObjectHeader},
    virtual_machine::cpu::{CPU, MemoryRecord},
};

use super::{IAccessor, ManagedReference};

impl ManagedReference<Class> {
    pub fn alloc_array<T>(cpu: &CPU, element_type: NonNull<MethodTable<T>>, len: usize) -> Self
    where
        TypeHandle: From<NonNull<T>>,
        T: GetValLayout,
    {
        let array_t = cpu
            .vm_ref()
            .assembly_manager()
            .get_core_type(CoreTypeId::System_Array_1)
            .unwrap_class();

        let array_t = unsafe { array_t.as_ref() };
        let instantiated_array_t =
            array_t.instantiate(&[MaybeUnloadedTypeHandle::Loaded(unsafe {
                TypeHandle::from(NonNull::from_ref(element_type.as_ref().ty_ref()))
            })]);

        let mut layout = Layout::new::<usize>();
        let element_layout = unsafe { element_type.as_ref().ty_ref().__get_val_layout() };
        (layout, _) = layout
            .extend(crate::memory::arrayed_layout(element_layout, len).unwrap())
            .unwrap();

        let full_layout = Self::calc_full_layout(layout).unwrap();
        let ptr = std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, full_layout).unwrap();
        unsafe {
            ptr.cast::<ObjectHeader>().write(ObjectHeader::new(false));
            ptr.byte_add(offset_of!(ManagedReferenceInner<Class>, mt))
                .cast::<NonNull<MethodTable<Class>>>()
                .write(*instantiated_array_t.as_ref().method_table());
        }

        let ptr = ptr.cast();
        let this = Self { data: Some(ptr) };

        cpu.push_record(MemoryRecord::new(
            NonGenericTypeHandleKind::Class,
            this.cast(),
        ))
        .unwrap();

        unsafe {
            this.data().unwrap().cast::<usize>().write(len);
        }

        this
    }

    /// The T's destructor will not be run, so it's recommended that T: Copy.
    pub fn new_array<TType, T: Copy + 'static>(
        cpu: &CPU,
        element_type: NonNull<MethodTable<TType>>,
        slice: Box<[T]>,
    ) -> Self
    where
        TypeHandle: From<NonNull<TType>>,
        TType: GetValLayout,
    {
        let array_t = cpu
            .vm_ref()
            .assembly_manager()
            .get_core_type(CoreTypeId::System_Array_1)
            .unwrap_class();

        let array_t = unsafe { array_t.as_ref() };
        let instantiated_array_t =
            array_t.instantiate(&[MaybeUnloadedTypeHandle::Loaded(unsafe {
                TypeHandle::from(NonNull::from_ref(element_type.as_ref().ty_ref()))
            })]);

        let mut layout = Layout::new::<usize>();
        let element_layout = unsafe { element_type.as_ref().ty_ref().__get_val_layout() };
        assert_eq!(element_layout, Layout::new::<T>());
        let element_offset;
        (layout, element_offset) = layout
            .extend(crate::memory::arrayed_layout(element_layout, slice.len()).unwrap())
            .unwrap();

        let full_layout = Self::calc_full_layout(layout).unwrap();
        let ptr = std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, full_layout).unwrap();
        unsafe {
            ptr.cast::<ObjectHeader>().write(ObjectHeader::new(false));
            ptr.byte_add(offset_of!(ManagedReferenceInner<Class>, mt))
                .cast::<NonNull<MethodTable<Class>>>()
                .write(*instantiated_array_t.as_ref().method_table());
        }

        let ptr = ptr.cast();
        let this = Self { data: Some(ptr) };

        cpu.push_record(MemoryRecord::new(
            NonGenericTypeHandleKind::Class,
            this.cast(),
        ))
        .unwrap();

        unsafe {
            this.data_ptr().cast::<usize>().write(slice.len());
            this.data_ptr()
                .byte_add(element_offset)
                .cast::<T>()
                .copy_from(slice.as_ptr(), slice.len());
            std::alloc::Allocator::deallocate(
                &std::alloc::Global,
                NonNull::new_unchecked(slice.as_ptr().cast_mut()).cast(),
                Layout::array::<T>(slice.len()).unwrap(),
            );
            std::mem::forget(slice);
        }

        this
    }

    pub fn is_array_like(&self) -> bool {
        self.method_table_ref().is_some_and(|mt| {
            mt.ty_ref().generic().is_some_and(|t| unsafe {
                let t_ref = t.as_ref();
                t_ref
                    .method_table_ref()
                    .get_core_type_id()
                    .is_some_and(|id| id == CoreTypeId::System_Array_1)
            })
        })
    }
}

#[repr(transparent)]
pub struct ArrayAccessor(ManagedReference<Class>);

impl<T> IAccessor<T> for ArrayAccessor {
    #[inline(always)]
    default fn is_valid(_: &ManagedReference<T>) -> bool {
        false
    }
}

impl IAccessor<Class> for ArrayAccessor {
    #[inline]
    fn is_valid(r: &ManagedReference<Class>) -> bool {
        r.is_array_like()
    }
}

#[inline(always)]
const fn precondition_check_const(_: &ArrayAccessor) {}

impl ArrayAccessor {
    pub const fn len(&self) -> Option<usize> {
        unsafe { Some(self.0.data()?.cast::<usize>().read()) }
    }
    /// # Safety
    /// self must be non-null
    pub const unsafe fn len_unchecked(&self) -> usize {
        unsafe { self.0.data_ptr().cast::<usize>().read() }
    }
    pub const fn is_empty(&self) -> bool {
        #[inline(always)]
        const fn is_zero(x: usize) -> bool {
            x.eq(&0)
        }
        self.len().is_none_or(is_zero)
    }

    /// Returns true if
    /// * The pointer(`self`) must be non-null.
    /// * The type must exist.
    fn can_get_element_type_handle(&self) -> bool {
        #[allow(clippy::borrowed_box)]
        const fn non_empty(x: &Box<[MaybeUnloadedTypeHandle]>) -> bool {
            !x.is_empty()
        }
        (!self.0.is_null())
            && unsafe {
                self.0
                    .method_table_ref_unchecked()
                    .ty_ref()
                    .type_vars()
                    .as_ref()
                    .is_some_and(non_empty)
            }
    }

    /// # Safety
    ///   * The pointer(`self`) must be non-null.
    ///   * The type must exist.
    ///
    ///   (i.e. [`Self::can_get_element_type_handle`] returns true)
    pub unsafe fn element_type_handle_unchecked(&self) -> TypeHandle {
        #[inline(always)]
        fn assert_get_element_type_handle(this: &ArrayAccessor) {
            debug_assert!(this.can_get_element_type_handle());
        }
        const_eval_select(
            (self,),
            precondition_check_const,
            assert_get_element_type_handle,
        );
        unsafe {
            let mt = self.0.method_table_ref_unchecked();
            let type_vars = mt.ty_ref().type_vars().as_ref().unwrap_unchecked();
            let t = type_vars.get_unchecked(0);
            t.load(mt.ty_ref().assembly_ref().manager_ref()).unwrap()
        }
    }
    pub fn element_type_handle(&self) -> Option<TypeHandle> {
        if self.can_get_element_type_handle() {
            unsafe { Some(self.element_type_handle_unchecked()) }
        } else {
            None
        }
    }

    pub fn element_layout(&self) -> Option<Layout> {
        self.element_type_handle().map(|th| {
            th.get_non_generic_with_type(unsafe { self.0.method_table_ref_unchecked().ty_ref() })
                .val_layout()
        })
    }

    fn check_element_layout<T>(&self) {
        let element_layout = self.element_layout().unwrap();
        debug_assert_eq!(element_layout, Layout::new::<T>());
    }

    /// # Safety
    /// T'size * len == data's layout
    pub const unsafe fn as_slice<T>(&self) -> Option<&[T]> {
        const_eval_select(
            (self,),
            precondition_check_const,
            Self::check_element_layout::<T>,
        );
        let len = self.len()?;
        unsafe {
            Some(std::slice::from_raw_parts(
                self.0
                    .data()?
                    .byte_add(size_of::<usize>())
                    .cast::<T>()
                    .as_ptr()
                    .cast_const(),
                len,
            ))
        }
    }
    /// # Safety
    /// T'size * len == data's layout && self is not null
    pub const unsafe fn as_slice_unchecked<T>(&self) -> &[T] {
        const_eval_select(
            (self,),
            precondition_check_const,
            Self::check_element_layout::<T>,
        );
        let len = unsafe { self.len_unchecked() };
        unsafe {
            std::slice::from_raw_parts(
                self.0
                    .data_ptr()
                    .byte_add(size_of::<usize>())
                    .cast::<T>()
                    .cast_const(),
                len,
            )
        }
    }

    /// # Safety
    /// T'size * len == data's layout
    pub const unsafe fn as_slice_mut<T>(&mut self) -> Option<&mut [T]> {
        const_eval_select(
            (&*self,),
            precondition_check_const,
            Self::check_element_layout::<T>,
        );
        let len = self.len()?;
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.0
                    .data()?
                    .byte_add(size_of::<usize>())
                    .cast::<T>()
                    .as_ptr(),
                len,
            ))
        }
    }
    /// # Safety
    /// T'size * len == data's layout && self is not null
    pub const unsafe fn as_slice_unchecked_mut<T>(&mut self) -> &mut [T] {
        const_eval_select(
            (&*self,),
            precondition_check_const,
            Self::check_element_layout::<T>,
        );
        let len = unsafe { self.len_unchecked() };
        unsafe {
            std::slice::from_raw_parts_mut(
                self.0.data_ptr().byte_add(size_of::<usize>()).cast::<T>(),
                len,
            )
        }
    }

    pub fn as_raw_slice(&self) -> Option<&[u8]> {
        let len = self.len()?;
        unsafe {
            Some(std::slice::from_raw_parts(
                self.0
                    .data()?
                    .byte_add(size_of::<usize>())
                    .cast::<u8>()
                    .as_ptr()
                    .cast_const(),
                len * self.element_layout()?.size(),
            ))
        }
    }

    pub fn as_raw_slice_mut(&self) -> Option<&mut [u8]> {
        let len = self.len()?;
        unsafe {
            Some(std::slice::from_raw_parts_mut(
                self.0
                    .data()?
                    .byte_add(size_of::<usize>())
                    .cast::<u8>()
                    .as_ptr(),
                len * self.element_layout()?.size(),
            ))
        }
    }

    pub fn as_raw_slices(&self) -> Option<std::slice::Chunks<'_, u8>> {
        let slice = self.as_raw_slice()?;

        Some(slice.chunks(self.element_layout()?.size()))
    }

    pub fn as_raw_slices_mut(&mut self) -> Option<std::slice::ChunksMut<'_, u8>> {
        let slice = self.as_raw_slice_mut()?;

        Some(slice.chunks_mut(self.element_layout()?.size()))
    }
}

#[cfg(test)]
mod tests {
    use crate::virtual_machine::{EnsureVirtualMachineInitialized, global_vm};

    use super::*;

    #[test]
    fn array_test() {
        EnsureVirtualMachineInitialized();
        let vm = global_vm();
        let cpu_id = vm.add_cpu();
        let cpu = vm.get_cpu(cpu_id).unwrap();

        let u8_t = *unsafe {
            vm.assembly_manager()
                .get_core_type(CoreTypeId::System_UInt8)
                .unwrap_struct()
                .as_ref()
                .method_table()
        };

        let mut arr = ManagedReference::alloc_array(&cpu, u8_t, 10);
        let arr_mut = unsafe {
            arr.access_mut::<ArrayAccessor>()
                .unwrap()
                .as_slice_mut::<u8>()
                .unwrap()
        };
        for (i, a) in arr_mut.iter_mut().enumerate() {
            *a = i as u8;
        }
        let arr_ref = unsafe {
            arr.access::<ArrayAccessor>()
                .unwrap()
                .as_slice::<u8>()
                .unwrap()
        };
        dbg!(arr_ref);
        arr.destroy(&cpu);

        let mut arr_by_new = ManagedReference::new_array::<_, u8>(
            &cpu,
            u8_t,
            Box::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        );
        let arr_by_new_ref = unsafe {
            arr_by_new
                .access::<ArrayAccessor>()
                .unwrap()
                .as_slice::<u8>()
                .unwrap()
        };
        dbg!(arr_by_new_ref);
        arr_by_new.destroy(&cpu);
    }
}
