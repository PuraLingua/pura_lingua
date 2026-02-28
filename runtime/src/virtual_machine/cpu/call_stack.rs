use std::{alloc::Layout, ptr::NonNull};

use enumflags2::BitFlags;
use global::{UnwrapEnum, attrs::MethodImplementationFlags, instruction::RegisterAddr};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
        method::{Method, MethodDisplayOptions},
        r#struct::Struct,
        type_handle::{NonGenericTypeHandle, NonGenericTypeHandleKind},
    },
    value::managed_reference::ManagedReference,
};

pub struct CallStack {
    stack: Vec<CallStackFrame>,
}

impl CallStack {
    pub const fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, frame: CallStackFrame) {
        self.stack.push(frame);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn current(&self) -> Option<&CallStackFrame> {
        self.stack.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut CallStackFrame> {
        self.stack.last_mut()
    }

    pub fn common_for_method<T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind>(
        &mut self,
        method: &Method<T>,
    ) {
        self.push(CallStackFrame::Common(
            CommonCallStackFrame::prepare_for_method(method),
        ));
    }

    pub fn push_native<T: GetNonGenericTypeHandleKind>(&mut self, method: &Method<T>) {
        self.push(CallStackFrame::native(method));
    }

    pub fn mark_all(&mut self) {
        for frame in &mut self.stack {
            frame.mark_all();
        }
    }

    pub fn capture(&self) -> impl Iterator<Item = &str> {
        #[inline]
        fn filter(frame: &&CallStackFrame) -> bool {
            !frame.should_hide_when_capturing()
        }
        self.stack
            .iter()
            .rev()
            .filter(filter)
            .map(CallStackFrame::name)
    }

    pub fn capture_with_options(
        &self,
        options: BitFlags<MethodDisplayOptions>,
    ) -> impl Iterator<Item = String> {
        #[inline]
        fn filter(frame: &&CallStackFrame) -> bool {
            !frame.should_hide_when_capturing()
        }
        self.stack
            .iter()
            .rev()
            .filter(filter)
            .map(move |x| match x.method() {
                (m, NonGenericTypeHandleKind::Class) => unsafe {
                    m.cast::<Method<Class>>()
                        .as_ref()
                        .display(options)
                        .to_string()
                },
                (m, NonGenericTypeHandleKind::Struct) => unsafe {
                    m.cast::<Method<Struct>>()
                        .as_ref()
                        .display(options)
                        .to_string()
                },
            })
    }
}

#[derive(UnwrapEnum)]
#[unwrap_enum(ref, ref_mut, owned, try)]
pub enum CallStackFrame {
    Native(NativeCallStackFrame),
    Common(CommonCallStackFrame),
}

impl CallStackFrame {
    pub const fn name(&self) -> &str {
        match self {
            Self::Native(native_call_stack_frame) => unsafe {
                native_call_stack_frame.method.as_ref().name()
            },
            Self::Common(common_call_stack_frame) => unsafe {
                common_call_stack_frame.method.as_ref().name()
            },
        }
    }

    pub const fn method(&self) -> (NonNull<Method<()>>, NonGenericTypeHandleKind) {
        match self {
            Self::Native(frame) => (frame.method, frame.kind),
            Self::Common(frame) => (frame.method, frame.kind),
        }
    }

    pub fn should_hide_when_capturing(&self) -> bool {
        match self {
            Self::Native(native_call_stack_frame) => {
                unsafe { native_call_stack_frame.method.as_ref() }
                    .attr()
                    .impl_flags()
                    .contains(MethodImplementationFlags::HideWhenCapturing)
            }
            Self::Common(common_call_stack_frame) => {
                unsafe { common_call_stack_frame.method.as_ref() }
                    .attr()
                    .impl_flags()
                    .contains(MethodImplementationFlags::HideWhenCapturing)
            }
        }
    }

    pub fn common_for_method<T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind>(
        method: &Method<T>,
    ) -> Self {
        Self::Common(CommonCallStackFrame::prepare_for_method(method))
    }
    pub const fn native<T: [const] GetNonGenericTypeHandleKind>(method: &Method<T>) -> Self {
        Self::Native(NativeCallStackFrame::new(method))
    }
}

impl CallStackFrame {
    pub fn mark_all(&mut self) {
        match self {
            Self::Native(frame) => frame.mark_all(),
            Self::Common(frame) => frame.mark_all(),
        }
    }
}

pub struct NativeCallStackFrame {
    pub(crate) method: NonNull<Method<()>>,
    pub(crate) kind: NonGenericTypeHandleKind,
    pub(crate) references: Vec<ManagedReference<()>>,
}

impl NativeCallStackFrame {
    pub const fn new<T: [const] GetNonGenericTypeHandleKind>(method: &Method<T>) -> Self {
        Self {
            method: NonNull::from_ref(method).cast(),
            kind: T::__get_non_generic_type_handle_kind(method.require_method_table_ref().ty_ref()),
            references: Vec::new(),
        }
    }
    pub fn with_capacity<T: GetNonGenericTypeHandleKind>(
        method: &Method<T>,
        capacity: usize,
    ) -> Self {
        Self {
            method: NonNull::from_ref(method).cast(),
            kind: T::__get_non_generic_type_handle_kind(method.require_method_table_ref().ty_ref()),
            references: Vec::with_capacity(capacity),
        }
    }
    pub fn mark_all(&mut self) {
        for r in &mut self.references {
            if let Some(header) = r.header_mut() {
                header.set_is_marked(true);
            }
        }
    }
}

pub struct LocalVariableInfo {
    pub offset: usize,
    pub layout: Layout,
    pub ty: NonGenericTypeHandle,
}

#[derive(Clone, Copy)]
pub struct LocalVariable {
    pub ptr: NonNull<u8>,
    pub layout: Layout,
    pub ty: NonGenericTypeHandle,
}

impl LocalVariable {
    pub fn as_ref_typed<'a, T>(&self) -> &'a T {
        debug_assert!(self.layout.size() >= size_of::<T>());
        unsafe { self.ptr.cast::<T>().as_ref() }
    }
    pub fn as_mut_typed<'a, T>(&mut self) -> &'a mut T {
        debug_assert!(self.layout.size() >= size_of::<T>());
        unsafe { self.ptr.cast::<T>().as_mut() }
    }
    #[inline]
    pub fn read_typed<T>(self) -> T {
        debug_assert!(self.layout.size() >= size_of::<T>());
        unsafe { self.ptr.cast::<T>().read() }
    }
    #[inline]
    pub fn write_typed<T>(self, val: T) {
        debug_assert!(self.layout.size() >= size_of::<T>());
        unsafe {
            self.ptr.cast::<T>().write(val);
        }
    }
    pub fn zero(self) {
        unsafe {
            self.ptr.write_bytes(0, self.layout.size());
        }
    }
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().cast_const(), self.layout.size()) }
    }
    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.layout.size()) }
    }
    pub fn is_all_zero(&self) -> bool {
        !self.as_bytes().iter().any(|x| x.ne(&0))
    }
    /// # Safety
    /// See [`std::ptr::NonNull::copy_from`]
    pub const unsafe fn copy_from(self, src: NonNull<u8>, count: usize) {
        #[cfg(debug_assertions)]
        fn check(this_size: usize, count: usize) {
            assert!(
                this_size >= count,
                "Requirement {count} too large(maximum: {this_size})"
            );
        }
        #[cfg(not(debug_assertions))]
        fn check(this: &LocalVariable, count: usize) {}
        const fn const_check(this_size: usize, count: usize) {
            assert!(this_size >= count);
        }
        core::intrinsics::const_eval_select((self.layout.size(), count), const_check, check);
        unsafe {
            self.ptr.copy_from(src, count);
        }
    }
    /// # Safety
    /// See [`std::ptr::NonNull::copy_to`]
    pub const unsafe fn copy_to(self, dest: NonNull<u8>, count: usize) {
        #[inline(always)]
        const fn noop_check(_: Layout, _: usize) {}
        cfg_select! {
            debug_assertions => {
                fn check(layout: Layout, count: usize) {
                    if layout.size() < count {
                        panic!(
                            "Size at most: {} while require {count} byte(s)",
                            layout.size()
                        )
                    }
                }
            }
            _ => {
                #[inline(always)]
                fn check(_: Layout, _: usize) {}
            }
        }
        std::intrinsics::const_eval_select((self.layout, count), noop_check, check);
        unsafe {
            self.ptr.copy_to(dest, count);
        }
    }
    /// # Safety
    /// See [`std::ptr::NonNull::copy_from`] where count = self.layout.size()
    pub const unsafe fn copy_all_from(self, src: NonNull<u8>) {
        unsafe {
            self.ptr.copy_from(src, self.layout.size());
        }
    }
    /// # Safety
    /// See [`std::ptr::NonNull::copy_to`] where count = self.layout.size()
    pub const unsafe fn copy_all_to(self, dest: NonNull<u8>) {
        unsafe {
            self.ptr.copy_to(dest, self.layout.size());
        }
    }
    pub const fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }
}

pub struct CommonCallStackFrame {
    method: NonNull<Method<()>>,
    kind: NonGenericTypeHandleKind,
    full_layout: Layout,
    layouts: Vec<LocalVariableInfo>,

    register_ptr: NonNull<u8>,
}

impl CommonCallStackFrame {
    pub fn new<T: GetNonGenericTypeHandleKind>(
        method: &Method<T>,
        full_layout: Layout,
        layouts: Vec<LocalVariableInfo>,
    ) -> Self {
        let register_ptr = if full_layout.size() == 0 {
            NonNull::dangling()
        } else {
            #[cfg(not(debug_assertions))]
            const ALLOCATE_FN: fn(
                &std::alloc::Global,
                Layout,
            ) -> Result<NonNull<[u8]>, std::alloc::AllocError> =
                <_ as std::alloc::Allocator>::allocate;

            #[cfg(debug_assertions)]
            const ALLOCATE_FN: fn(
                &std::alloc::Global,
                Layout,
            ) -> Result<NonNull<[u8]>, std::alloc::AllocError> =
                <_ as std::alloc::Allocator>::allocate_zeroed;
            ALLOCATE_FN(&std::alloc::Global, full_layout)
                .unwrap()
                .as_non_null_ptr()
        };

        Self {
            method: NonNull::from_ref(method).cast(),
            kind: T::__get_non_generic_type_handle_kind(method.require_method_table_ref().ty_ref()),
            full_layout,
            layouts,
            register_ptr,
        }
    }

    pub fn prepare_for_method<T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind>(
        method: &Method<T>,
    ) -> Self {
        let types = method.attr().local_variable_types();
        let ty_ref = method.require_method_table_ref().ty_ref();

        let mut full_layout = Layout::new::<()>();
        let mut infos = Vec::with_capacity(types.len());

        for ty in types {
            let ty = ty
                .load(ty_ref.__get_assembly_ref().manager_ref())
                .unwrap()
                .get_non_generic_with_method(method);
            let layout = ty.val_layout();
            let offset;
            (full_layout, offset) = full_layout.extend(layout).unwrap();
            infos.push(LocalVariableInfo { offset, layout, ty });
        }

        Self::new(method, full_layout, infos)
    }

    pub fn get(&self, i: RegisterAddr) -> Option<LocalVariable> {
        self.layouts
            .get(i.get_usize())
            .map(|&LocalVariableInfo { offset, layout, ty }| LocalVariable {
                ptr: unsafe { self.register_ptr.byte_add(offset) },
                layout,
                ty,
            })
    }

    pub fn get_typed<T>(&self, i: RegisterAddr) -> Option<&T> {
        self.get(i).map(|x| LocalVariable::as_ref_typed::<T>(&x))
    }

    pub fn read_typed<T>(&self, i: RegisterAddr) -> Option<T> {
        self.get(i).map(LocalVariable::read_typed)
    }
    /// Return false if i is not found
    pub fn write_typed<T>(&self, i: RegisterAddr, val: T) -> bool {
        match self.get(i) {
            None => false,
            Some(local_var) => {
                local_var.write_typed(val);
                true
            }
        }
    }

    /// Return false if i is not found
    pub fn zero_register(&self, i: RegisterAddr) -> bool {
        match self.get(i) {
            None => false,
            Some(local_var) => {
                local_var.zero();
                true
            }
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.register_ptr.as_ptr().cast_const(),
                self.full_layout.size(),
            )
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = LocalVariable> {
        CommonCallStackFrameIter {
            this: self,
            index: RegisterAddr::ZERO,
        }
    }

    pub fn info_iter(&self) -> impl Iterator<Item = &LocalVariableInfo> {
        CommonCallStackFrameInfoIter {
            this: self,
            index: RegisterAddr::ZERO,
        }
    }

    pub fn mark_all(&mut self) {
        for mut var in self.iter().filter(|x| x.ty.is_managed_reference()) {
            if let Some(header) = var.as_mut_typed::<ManagedReference<()>>().header_mut() {
                header.set_is_marked(true);
            }
        }
    }
}

pub struct CommonCallStackFrameIter<'a> {
    this: &'a CommonCallStackFrame,
    index: RegisterAddr,
}

pub struct CommonCallStackFrameInfoIter<'a> {
    this: &'a CommonCallStackFrame,
    index: RegisterAddr,
}

impl<'a> Iterator for CommonCallStackFrameIter<'a> {
    type Item = LocalVariable;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.this.get(self.index)?;
        self.index += 1;
        Some(result)
    }
}

impl<'a> Iterator for CommonCallStackFrameInfoIter<'a> {
    type Item = &'a LocalVariableInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.this.layouts.get(self.index.get_usize())?;
        self.index += 1;

        Some(result)
    }
}

impl Drop for CommonCallStackFrame {
    fn drop(&mut self) {
        if self.full_layout.size() != 0 {
            unsafe {
                std::alloc::Allocator::deallocate(
                    &std::alloc::Global,
                    self.register_ptr,
                    self.full_layout,
                );
            }
        }
    }
}

mod getset_cpu {
    use std::sync::{
        LockResult, MappedRwLockReadGuard, MappedRwLockWriteGuard, PoisonError, RwLockReadGuard,
        RwLockWriteGuard,
    };

    use enumflags2::BitFlags;

    use crate::type_system::{
        get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
        method::{Method, MethodDisplayOptions},
    };

    use super::{super::CPU, CallStack, CallStackFrame, CommonCallStackFrame};

    impl CPU {
        pub fn read_call_stack(&self) -> LockResult<RwLockReadGuard<'_, CallStack>> {
            self.call_stack.read()
        }
        pub fn write_call_stack(&self) -> LockResult<RwLockWriteGuard<'_, CallStack>> {
            self.call_stack.write()
        }
        pub fn push_call_stack(
            &self,
            frame: CallStackFrame,
        ) -> Result<(), PoisonError<RwLockWriteGuard<'_, CallStack>>> {
            let mut call_stack = self.write_call_stack()?;
            call_stack.push(frame);

            Ok(())
        }
        pub fn push_call_stack_native<T: GetNonGenericTypeHandleKind>(
            &self,
            method: &Method<T>,
        ) -> Result<(), PoisonError<RwLockWriteGuard<'_, CallStack>>> {
            self.push_call_stack(CallStackFrame::native(method))
        }
        pub fn pop_call_stack(&self) -> Result<(), PoisonError<RwLockWriteGuard<'_, CallStack>>> {
            let mut call_stack = self.write_call_stack()?;
            call_stack.pop();

            Ok(())
        }
        pub fn prepare_call_stack_for_method<
            T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind,
        >(
            &self,
            method: &Method<T>,
        ) -> Result<(), PoisonError<RwLockWriteGuard<'_, CallStack>>> {
            self.push_call_stack(CallStackFrame::common_for_method(method))
        }
        pub fn current_call_frame<'a>(
            &'a self,
        ) -> Result<
            Option<MappedRwLockReadGuard<'a, CallStackFrame>>,
            PoisonError<RwLockReadGuard<'a, CallStack>>,
        > {
            self.read_call_stack()
                .map(|x| RwLockReadGuard::filter_map(x, |x| x.current()).ok())
        }
        pub fn current_call_frame_mut<'a>(
            &'a self,
        ) -> Result<
            Option<MappedRwLockWriteGuard<'a, CallStackFrame>>,
            PoisonError<RwLockWriteGuard<'a, CallStack>>,
        > {
            self.write_call_stack()
                .map(|x| RwLockWriteGuard::filter_map(x, |x| x.current_mut()).ok())
        }
        pub fn current_common_call_frame<'a>(
            &'a self,
        ) -> Result<
            Option<MappedRwLockReadGuard<'a, CommonCallStackFrame>>,
            PoisonError<RwLockReadGuard<'a, CallStack>>,
        > {
            self.current_call_frame().map(|x| {
                x.and_then(|x| {
                    MappedRwLockReadGuard::filter_map(x, |x| x.unwrap_common_ref().ok()).ok()
                })
            })
        }
        pub fn current_common_call_frame_mut<'a>(
            &'a self,
        ) -> Result<
            Option<MappedRwLockWriteGuard<'a, CommonCallStackFrame>>,
            PoisonError<RwLockWriteGuard<'a, CallStack>>,
        > {
            self.current_call_frame_mut().map(|x| {
                x.and_then(|x| {
                    MappedRwLockWriteGuard::filter_map(x, |x| x.unwrap_common_mut().ok()).ok()
                })
            })
        }
        pub fn capture<'a>(
            &'a self,
        ) -> Result<Vec<String>, PoisonError<RwLockReadGuard<'a, CallStack>>> {
            Ok(self
                .read_call_stack()?
                .capture()
                .map(ToOwned::to_owned)
                .collect())
        }
        pub fn capture_with_options(
            &self,
            options: BitFlags<MethodDisplayOptions>,
        ) -> Result<Vec<String>, PoisonError<RwLockReadGuard<'_, CallStack>>> {
            Ok(self
                .read_call_stack()?
                .capture_with_options(options)
                .collect())
        }
    }
}
