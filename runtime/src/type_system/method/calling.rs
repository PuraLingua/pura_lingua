use std::{
    alloc::{Allocator, Layout},
    ffi::c_void,
    ptr::NonNull,
};

use enumflags2::BitFlags;
use global::attrs::CallConvention;

use crate::{
    type_system::get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
    virtual_machine::cpu::CPU,
};

use super::{Method, default_entry_point};

impl<T: GetTypeVars + GetAssemblyRef + GetNonGenericTypeHandleKind> Method<T> {
    fn get_cif(&self) -> libffi::middle::Cif {
        use libffi::middle::{Builder, Type};
        let mut builder = Builder::new()
            .abi(self.libffi_call_convention())
            .res(self.libffi_return_type())
            .args([
                /* CPU */ Type::pointer(),
                /* Method */ Type::pointer(),
            ]);
        if !self.attr.is_static() {
            builder = builder.arg(Type::pointer());
        }

        for param in self.args() {
            builder = builder.arg(param.get_libffi_type(self));
        }

        builder.into_cif()
    }

    fn handle_args(
        self: &&Self,
        cpu: &&CPU,
        this: Option<&NonNull<()>>,
        args: &[*mut c_void],
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
        if self.call_convention() != CallConvention::CDeclWithVararg {
            assert_eq!(args.len(), self.args.len());
        }

        for (ind, a) in args.iter().enumerate() {
            if !self.args[ind].attr.is_by_ref() {
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
                complete_arg.push(*a);
            } else {
                complete_arg.push((&raw const *a).cast_mut().cast());
            }
        }

        complete_arg
    }

    pub fn untyped_call(
        &self,
        cpu: &CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
    ) -> (NonNull<u8>, Layout) {
        println!("Calling Method: {}", self.display(BitFlags::all()));

        if std::ptr::addr_eq(
            default_entry_point::__default_entry_point::<T> as *const c_void,
            self.entry_point.as_ptr(),
        ) {
            cpu.prepare_call_stack_for_method(self).unwrap();
            let res = default_entry_point::__default_entry_point::<T>(self, cpu, this, args);
            cpu.pop_call_stack().unwrap();
            return res;
        }
        cpu.push_call_stack_native(self).unwrap();
        let cif = self.get_cif();

        let mut args = self.handle_args(&cpu, this.as_ref(), args);

        let mut ret_layout = self.get_return_type().val_layout();
        if ret_layout.size() < size_of::<usize>() {
            ret_layout = Layout::new::<usize>();
        }

        let result =
            std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, ret_layout).unwrap();
        unsafe {
            libffi::raw::ffi_call(
                cif.as_raw_ptr(),
                Some(*self.entry_point.as_safe_fun()),
                result.as_non_null_ptr().cast::<c_void>().as_ptr(),
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

        cpu.pop_call_stack().unwrap();

        (result.as_non_null_ptr(), ret_layout)
    }

    pub fn typed_res_call<R>(
        &self,
        cpu: &CPU,
        this: Option<NonNull<()>>,
        args: &[*mut std::ffi::c_void],
    ) -> R {
        let (ret_ptr, ret_layout) = self.untyped_call(cpu, this, args);
        let res = unsafe { ret_ptr.cast::<R>().read() };
        unsafe {
            Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
        }

        res
    }

    pub fn typed_call<'a, R>(
        &self,
        cpu: &CPU,
        this: Option<NonNull<()>>,
        args: &[libffi::middle::Arg<'a>],
    ) -> R {
        let (ret_ptr, ret_layout) = self.untyped_call(cpu, this, unsafe {
            &*(args as *const [_] as *const [*mut c_void])
        });
        let res = unsafe { ret_ptr.cast::<R>().read() };
        unsafe {
            Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
        }

        res
    }
}
