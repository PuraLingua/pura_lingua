use std::{alloc::Layout, ptr::NonNull};

use enumflags2::BitFlags;
use global::{UnwrapEnum, attrs::MethodImplementationFlags};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
        method::{Method, MethodDisplayOptions},
        r#struct::Struct,
        type_handle::NonGenericTypeHandleKind,
    },
    value::managed_reference::ManagedReference,
};

use super::CPU;

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

    pub fn capture<'a>(&self) -> impl Iterator<Item = &str> {
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

    pub fn capture_with_options<'a>(
        &'a self,
        options: BitFlags<MethodDisplayOptions>,
    ) -> impl Iterator<Item = String> + use<'a> {
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
}

pub struct CommonCallStackFrame {
    method: NonNull<Method<()>>,
    kind: NonGenericTypeHandleKind,
    full_layout: Layout,
    layouts: Vec<(usize, Layout)>,

    register_ptr: NonNull<u8>,
}

impl CommonCallStackFrame {
    pub fn new<T: GetNonGenericTypeHandleKind>(
        method: &Method<T>,
        full_layout: Layout,
        layouts: Vec<(usize, Layout)>,
    ) -> Self {
        let register_ptr = if full_layout.size() == 0 {
            NonNull::dangling()
        } else {
            std::alloc::Allocator::allocate(&std::alloc::Global, full_layout)
                .unwrap()
                .cast()
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
        let mut layouts = Vec::with_capacity(types.len());

        for ty in types {
            let layout = ty
                .load(ty_ref.__get_assembly_ref().manager_ref())
                .unwrap()
                .val_layout_with_type(ty_ref);
            let offset;
            (full_layout, offset) = full_layout.extend(layout).unwrap();
            layouts.push((offset, layout));
        }

        Self::new(method, full_layout, layouts)
    }

    pub fn get(&self, i: u64) -> Option<(NonNull<u8>, Layout)> {
        self.layouts
            .get(i as usize)
            .map(|(offset, layout)| unsafe { (self.register_ptr.byte_add(*offset), *layout) })
    }

    pub fn get_typed<T>(&self, i: u64) -> Option<&T> {
        self.get(i).map(|(p, l)| {
            debug_assert!(l.size() >= size_of::<T>());
            unsafe { p.cast::<T>().as_ref() }
        })
    }

    /// Return false if i is not found
    pub fn write_typed<T>(&self, i: u64, v: T) -> bool {
        match self.get(i) {
            None => false,
            Some((p, l)) => {
                debug_assert!(l.size() >= size_of::<T>());
                unsafe {
                    p.cast::<T>().write(v);
                }
                true
            }
        }
    }

    /// Return false if i is not found
    pub fn zero_register(&self, i: u64) -> bool {
        match self.get(i) {
            None => false,
            Some((p, l)) => {
                unsafe {
                    p.write_bytes(0, l.size());
                }
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
