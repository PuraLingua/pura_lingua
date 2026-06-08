use std::{alloc::Layout, ffi::c_void, mem::MaybeUninit, ptr::NonNull};

use global::attrs::{CallConvention, MethodImplementationFlags};

use crate::{
    type_system::get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
    virtual_machine::cpu::CPU,
};

use super::{Method, default_entry_point};

impl<T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind> Method<T> {
    fn get_cif(&self) -> libffi::middle::Cif {
        use libffi::middle::{Builder, Type};
        let mut builder = Builder::new().abi(self.libffi_call_convention()).args([
            /* CPU */ Type::pointer(),
            /* Method */ Type::pointer(),
        ]);

        if !self.attr.is_static() {
            builder = builder.arg(Type::pointer());
        }

        for param in self.args() {
            builder = builder.arg(param.get_libffi_type(self));
        }

        if self.attr.allow_extra_args() {
            builder = builder
                .arg(/* Pointer to extra args */ Type::pointer())
                .arg(/* extra arg length */ Type::usize());
        }

        if self
            .attr
            .impl_flags
            .contains(MethodImplementationFlags::UseReturnBuffer)
        {
            builder = builder.res(libffi::middle::Type::void());
            builder = builder.arg(Type::pointer());
        } else {
            builder = builder.res(self.libffi_return_type());
        }

        builder.into_cif()
    }

    fn handle_args(
        self: &&Self,
        cpu: &&CPU,
        this: Option<&NonNull<()>>,
        rest_arg_ptr: &mut NonNull<*mut c_void>,
        rest_arg_len: &mut usize,
        args: &[*mut c_void],
        return_buffer: &mut NonNull<c_void>,
    ) -> Vec<*mut c_void> {
        // It will be either 0 or 1
        let this_arg_len = if self.attr.is_static() { 0 } else { 1 };
        let mut complete_arg = Vec::with_capacity(3 + this_arg_len + args.len());
        complete_arg.push((&raw const *cpu).cast::<std::ffi::c_void>().cast_mut());
        complete_arg.push((&raw const *self).cast::<std::ffi::c_void>().cast_mut());
        if !self.attr.is_static() {
            let this = this.unwrap();
            complete_arg.push((&raw const *this).cast_mut().cast());
        }

        #[cfg(debug_assertions)]
        if (self.call_convention() != CallConvention::CDeclWithVararg)
            && (!self.attr.allow_extra_args())
        {
            assert_eq!(args.len(), self.args.len());
        }

        for (ind, param) in self.args.iter().enumerate() {
            if !param.attr.is_by_ref() {
                // let layout = self.args[ind].get_layout(self);
                // if layout.size() == 0 {
                //     complete_arg.push(NonNull::dangling().as_ptr());
                //     continue;
                // }
                // copied_args.push((ind + 2 + this_arg_len, layout));
                // let ptr = std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, layout)
                //     .unwrap()
                //     .as_non_null_ptr()
                //     .cast::<c_void>();

                // if let Some(a) = NonNull::new(*a) {
                //     unsafe {
                //         ptr.cast::<u8>().copy_from(a.cast(), layout.size());
                //     }
                // }
                complete_arg.push(args[ind]);
            } else {
                complete_arg.push((&raw const args[ind]).cast_mut().cast());
            }
        }

        if self.attr.allow_extra_args() {
            *rest_arg_len = args.len() - self.args.len();
            *rest_arg_ptr = if (*rest_arg_len) == 0 {
                NonNull::dangling()
            } else {
                NonNull::from_ref(unsafe { args.get_unchecked(self.args.len()) })
            };
            complete_arg.push((&raw mut *rest_arg_ptr).cast());
            complete_arg.push((&raw mut *rest_arg_len).cast());
        }

        if self
            .attr
            .impl_flags
            .contains(MethodImplementationFlags::UseReturnBuffer)
        {
            complete_arg.push((&raw mut *return_buffer).cast());
        }

        complete_arg
    }

    pub unsafe fn buffered_call(
        &self,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        mut return_buffer: NonNull<c_void>,
    ) {
        #[cfg(feature = "print_invoke_and_call")]
        println!(
            "Calling Method: {}",
            self.display(enumflags2::BitFlags::all())
        );

        unsafe {
            return_buffer.write_bytes(
                0,
                crate::memory::get_return_layout_for_libffi(self.get_return_type().val_layout())
                    .size(),
            );
        }

        if std::ptr::addr_eq(
            default_entry_point::__default_entry_point::<T> as *const c_void,
            self.entry_point.as_ptr(),
        ) {
            cpu.prepare_call_stack_for_method(self);
            default_entry_point::__default_entry_point::<T>(self, cpu, this, args, return_buffer);
            cpu.pop_call_stack();
            return;
        }
        cpu.push_call_stack_native(self);
        let cif = self.get_cif();

        let mut rest_arg_ptr = NonNull::dangling();
        let mut rest_arg_len = 0;

        let mut args = self.handle_args(
            &&*cpu,
            this.as_ref(),
            &mut rest_arg_ptr,
            &mut rest_arg_len,
            args,
            &mut return_buffer,
        );

        unsafe {
            libffi::raw::ffi_call(
                cif.as_raw_ptr(),
                Some(*self.entry_point.as_safe_fun()),
                return_buffer.as_ptr(),
                args.as_mut_ptr(),
            );
        }

        // for (ca_index, ca_layout) in copied_args {
        //     if let Some(ca_ptr) = NonNull::new(args[ca_index]) {
        //         unsafe {
        //             Allocator::deallocate(&std::alloc::Global, ca_ptr.cast(), ca_layout);
        //         }
        //     }
        // }

        cpu.pop_call_stack();
    }

    pub fn untyped_call(
        &self,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
    ) -> (NonNull<u8>, Layout) {
        let ret_layout =
            crate::memory::get_return_layout_for_libffi(self.get_return_type().val_layout());

        let result =
            std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, ret_layout).unwrap();

        unsafe {
            self.buffered_call(cpu, this, args, result.as_non_null_ptr().cast());
        }

        (result.as_non_null_ptr(), ret_layout)
    }

    pub fn typed_res_call<R>(
        &self,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut std::ffi::c_void],
    ) -> R {
        #[cfg(debug_assertions)]
        {
            let return_layout = self.get_return_type().val_layout();
            assert_eq!(return_layout, Layout::new::<R>());
        }

        const fn check<R>() {
            if size_of::<R>() < size_of::<usize>() {
                assert!(align_of::<R>() <= align_of::<usize>());
            }
        }

        const { check::<R>() }

        // See
        const _: unsafe fn(*mut libffi::raw::ffi_cif, libffi::low::CodePtr, *mut *mut c_void) =
            libffi::low::call::<()>;
        if size_of::<R>() < size_of::<usize>() {
            #[cfg(not(debug_assertions))]
            let mut result: MaybeUninit<usize> = MaybeUninit::uninit();
            #[cfg(debug_assertions)]
            let mut result: MaybeUninit<usize> = MaybeUninit::zeroed();

            unsafe {
                self.buffered_call(
                    cpu,
                    this,
                    args,
                    NonNull::new_unchecked(result.as_mut_ptr().cast()),
                );
            }

            if cfg!(target_endian = "big") {
                // See [libffi::low::call_return_small_big_endian_result]

                let type_tag =
                    unsafe { (*self.get_return_type().val_libffi_type().as_raw_ptr()).type_ };

                if [
                    libffi::raw::FFI_TYPE_FLOAT,
                    libffi::raw::FFI_TYPE_STRUCT,
                    libffi::raw::FFI_TYPE_VOID,
                ]
                .contains(&type_tag)
                {
                    // SAFETY: Testing has shown that these types appear at `result`.
                    unsafe { result.as_ptr().cast::<R>().read() }
                } else {
                    // SAFETY: Consider `*result` an array with
                    // `size_of::<usize>() / size_of::<R>()` items of `R`. The following
                    // code reads the last element to get the least significant bits of
                    // `result` on big endian architectures. The most significant bits are
                    // zeroed by libffi.
                    unsafe {
                        result
                            .as_ptr()
                            .cast::<R>()
                            .add((size_of::<usize>() / size_of::<R>()) - 1)
                            .read()
                    }
                }
            } else {
                unsafe { result.as_ptr().cast::<R>().read() }
            }
        } else {
            #[cfg(not(debug_assertions))]
            let mut result: MaybeUninit<R> = MaybeUninit::uninit();
            #[cfg(debug_assertions)]
            let mut result: MaybeUninit<R> = MaybeUninit::zeroed();

            unsafe {
                self.buffered_call(
                    cpu,
                    this,
                    args,
                    NonNull::new_unchecked(result.as_mut_ptr().cast()),
                );
            }

            unsafe { result.assume_init() }
        }
    }

    #[inline(always)]
    pub fn typed_call<'a, R>(
        &self,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[libffi::middle::Arg<'a>],
    ) -> R {
        self.typed_res_call(cpu, this, unsafe {
            &*(args as *const [_] as *const [*mut c_void])
        })
    }
}
