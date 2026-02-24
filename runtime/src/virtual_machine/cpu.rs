use std::{
    alloc::Layout,
    ffi::c_void,
    mem::DropGuard,
    process::Termination,
    ptr::{NonNull, Unique},
    sync::{LockResult, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    error::RuntimeError,
    stdlib::{
        CoreTypeId, System_NonPurusCallConfiguration_FieldId, System_NonPurusCallType_FieldId,
    },
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
        method::{Method, MethodRef},
        type_handle::{MaybeUnloadedTypeHandle, TypeHandle},
    },
    value::managed_reference::{ArrayAccessor, FieldAccessor, ManagedReference},
};

use super::VirtualMachine;

mod call_stack;
mod exception;
mod gc;
mod mem_record;

pub use call_stack::{CallStack, CallStackFrame, CommonCallStackFrame, NativeCallStackFrame};
pub use exception::{ExceptionManager, ThrowHelper, errno_location};
use global::{
    attrs::CallConvention,
    dt_println,
    non_purus_call_configuration::{NonPurusCallConfiguration, NonPurusCallType},
};
use line_ending::LineEnding;
pub use mem_record::MemoryRecord;

#[cfg(test)]
mod tests;

pub struct CPU {
    vm: NonNull<VirtualMachine>,

    call_stack: RwLock<CallStack>,

    mem_records: RwLock<Vec<MemoryRecord>>,
    exception_manager: RwLock<ExceptionManager>,
}

impl CPU {
    pub fn new(vm: NonNull<VirtualMachine>) -> Unique<Self> {
        let this = Box::new(Self {
            vm,
            call_stack: RwLock::new(CallStack::new()),
            mem_records: RwLock::new(Vec::new()),
            exception_manager: RwLock::new(ExceptionManager::new()),
        });

        Unique::from_non_null(Box::into_non_null(this))
    }
}

impl CPU {
    pub fn vm_ref(&self) -> &VirtualMachine {
        unsafe { self.vm.as_ref() }
    }
    #[track_caller]
    pub fn read_mem_records(&self) -> LockResult<RwLockReadGuard<'_, Vec<MemoryRecord>>> {
        self.mem_records.read()
    }
    #[track_caller]
    pub fn push_record(
        &self,
        record: MemoryRecord,
    ) -> Result<(), PoisonError<RwLockWriteGuard<'_, Vec<MemoryRecord>>>> {
        self.mem_records.write()?.push(record);

        Ok(())
    }
    #[track_caller]
    pub fn write_mem_records(&self) -> LockResult<RwLockWriteGuard<'_, Vec<MemoryRecord>>> {
        self.mem_records.write()
    }
}

#[derive(Debug)]
pub enum MainResult {
    Void,
    VoidWithException,
    Custom(u8),
    SignatureNotMatched,
}

impl std::process::Termination for MainResult {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Void => std::process::ExitCode::SUCCESS,
            Self::VoidWithException => std::process::ExitCode::FAILURE,
            Self::Custom(code) => std::process::ExitCode::from(code),
            Self::SignatureNotMatched => {
                eprintln!("The signature of the Main method is mismatched.");
                std::process::ExitCode::FAILURE
            }
        }
    }
}

impl CPU {
    pub fn marshal_non_purus_type(&self, ty: &NonPurusCallType) -> ManagedReference<Class> {
        let mt = unsafe {
            self.vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_NonPurusCallType)
                .unwrap_class()
                .as_ref()
                .method_table
        };
        let mut obj = ManagedReference::common_alloc(self, mt, false);
        *obj.const_access_mut::<FieldAccessor<Class>>()
            .typed_field_mut(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
            )
            .unwrap() = ty.discriminant();
        match ty {
            NonPurusCallType::Structure(fields) => {
                let arr = ManagedReference::new_array::<_, ManagedReference<Class>>(
                    self,
                    mt,
                    fields
                        .iter()
                        .map(|x| self.marshal_non_purus_type(x))
                        .collect(),
                );
                *obj.const_access_mut::<FieldAccessor<Class>>()
                    .typed_field_mut(
                        System_NonPurusCallType_FieldId::Types as _,
                        Default::default(),
                    )
                    .unwrap() = arr;
            }
            _ => (),
        }
        obj
    }
    pub fn marshal_non_purus_configuration(
        &self,
        cfg: &NonPurusCallConfiguration,
    ) -> ManagedReference<Class> {
        let mt = unsafe {
            self.vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_NonPurusCallConfiguration)
                .unwrap_class()
                .as_ref()
                .method_table
        };
        let usize_mt = unsafe {
            self.vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_USize)
                .unwrap_struct()
                .as_ref()
                .method_table
        };
        let type_mt = unsafe {
            self.vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_NonPurusCallType)
                .unwrap_class()
                .as_ref()
                .method_table
        };

        let mut obj = ManagedReference::common_alloc(self, mt, false);
        let obj_fields = obj.const_access_mut::<FieldAccessor<Class>>();

        assert!(obj_fields.write_typed_field::<u8>(
            System_NonPurusCallConfiguration_FieldId::CallConvention as _,
            Default::default(),
            cfg.call_convention.into()
        ));

        assert!(obj_fields.write_typed_field::<ManagedReference<Class>>(
            System_NonPurusCallConfiguration_FieldId::ReturnType as _,
            Default::default(),
            self.marshal_non_purus_type(&cfg.return_type)
        ));

        assert!(obj_fields.write_typed_field::<u8>(
            System_NonPurusCallConfiguration_FieldId::Encoding as _,
            Default::default(),
            cfg.encoding.into(),
        ));

        assert!(obj_fields.write_typed_field::<u8>(
            System_NonPurusCallConfiguration_FieldId::ObjectStrategy as _,
            Default::default(),
            cfg.object_strategy.into()
        ));

        let mut by_ref_args = Vec::new();
        let mut args = Vec::new();
        for (i, (is_by_ref, arg)) in cfg.arguments.iter().enumerate() {
            args.push(self.marshal_non_purus_type(arg));
            if *is_by_ref {
                by_ref_args.push(i);
            }
        }
        let by_ref_args = by_ref_args.into_boxed_slice();
        let args = args.into_boxed_slice();
        let m_by_ref_args = ManagedReference::new_array(self, usize_mt, by_ref_args);
        let m_args = ManagedReference::new_array(self, type_mt, args);
        assert!(obj_fields.write_typed_field::<ManagedReference<Class>>(
            System_NonPurusCallConfiguration_FieldId::ByRefArguments as _,
            Default::default(),
            m_by_ref_args,
        ));
        assert!(obj_fields.write_typed_field::<ManagedReference<Class>>(
            System_NonPurusCallConfiguration_FieldId::Arguments as _,
            Default::default(),
            m_args,
        ));

        obj
    }
    pub fn unmarshal_non_purus_type(ty: ManagedReference<Class>) -> Option<NonPurusCallType> {
        let accessor = ty.const_access::<FieldAccessor<Class>>();
        let discriminant = accessor.read_typed_field::<u8>(
            System_NonPurusCallType_FieldId::Discriminant as _,
            Default::default(),
        )?;
        NonPurusCallType::from_u8(discriminant, || {
            let field_types = accessor.typed_field::<ManagedReference<Class>>(
                System_NonPurusCallType_FieldId::Types as _,
                Default::default(),
            )?;
            unsafe {
                field_types
                    .access::<ArrayAccessor>()?
                    .as_slice::<ManagedReference<Class>>()?
            }
            .iter()
            .copied()
            .map(Self::unmarshal_non_purus_type)
            .try_collect()
        })
    }
    pub fn unmarshal_non_purus_configuration(
        &self,
        cfg: ManagedReference<Class>,
    ) -> global::Result<NonPurusCallConfiguration> {
        let accessor = cfg.const_access::<FieldAccessor<Class>>();

        let call_convention: CallConvention = CallConvention::try_from(
            accessor
                .read_typed_field::<u8>(
                    System_NonPurusCallConfiguration_FieldId::CallConvention as _,
                    Default::default(),
                )
                .ok_or(RuntimeError::FailedGetField(
                    System_NonPurusCallConfiguration_FieldId::CallConvention as _,
                ))?,
        )?;

        let return_type = Self::unmarshal_non_purus_type(
            accessor
                .read_typed_field::<ManagedReference<Class>>(
                    System_NonPurusCallConfiguration_FieldId::ReturnType as _,
                    Default::default(),
                )
                .ok_or(RuntimeError::FailedGetField(
                    System_NonPurusCallConfiguration_FieldId::ReturnType as _,
                ))?,
        )
        .ok_or(RuntimeError::unmarshal_failed::<NonPurusCallType>())?;

        let encoding = global::non_purus_call_configuration::StringEncoding::try_from(
            accessor
                .read_typed_field::<u8>(
                    System_NonPurusCallConfiguration_FieldId::Encoding as _,
                    Default::default(),
                )
                .ok_or(RuntimeError::FailedGetField(
                    System_NonPurusCallConfiguration_FieldId::Encoding as _,
                ))?,
        )?;

        let object_strategy = global::non_purus_call_configuration::ObjectStrategy::try_from(
            accessor
                .read_typed_field::<u8>(
                    System_NonPurusCallConfiguration_FieldId::ObjectStrategy as _,
                    Default::default(),
                )
                .ok_or(RuntimeError::FailedGetField(
                    System_NonPurusCallConfiguration_FieldId::ObjectStrategy as _,
                ))?,
        )?;

        let by_ref_args = accessor
            .read_typed_field::<ManagedReference<Class>>(
                System_NonPurusCallConfiguration_FieldId::ByRefArguments as _,
                Default::default(),
            )
            .ok_or(RuntimeError::FailedGetField(
                System_NonPurusCallConfiguration_FieldId::ByRefArguments as _,
            ))?;

        let in_sign_args = accessor
            .read_typed_field::<ManagedReference<Class>>(
                System_NonPurusCallConfiguration_FieldId::Arguments as _,
                Default::default(),
            )
            .ok_or(RuntimeError::FailedGetField(
                System_NonPurusCallConfiguration_FieldId::Arguments as _,
            ))?;

        let mut unmarshaled_args = unsafe {
            in_sign_args
                .access::<ArrayAccessor>()
                .ok_or(RuntimeError::NotAnArray)?
                .as_slice::<ManagedReference<Class>>()
                .ok_or(global::errors::NullPointerError)?
        }
        .iter()
        .copied()
        .map(Self::unmarshal_non_purus_type)
        .map(|x| {
            x.map(|x| (false, x))
                .ok_or(RuntimeError::unmarshal_failed::<NonPurusCallType>())
        })
        .try_collect::<Vec<_>>()?;

        for enable_by_ref in unsafe {
            by_ref_args
                .access::<ArrayAccessor>()
                .ok_or(RuntimeError::NotAnArray)?
                .as_slice::<usize>()
                .ok_or(global::errors::NullPointerError)?
        } {
            unmarshaled_args[*enable_by_ref].0 = true;
        }

        Ok(NonPurusCallConfiguration {
            call_convention,
            return_type,
            encoding,
            object_strategy,
            arguments: unmarshaled_args,
        })
    }
    pub fn dynamic_non_purus_call(
        &self,
        cfg: ManagedReference<Class>,
        f_ptr: *const u8,
        args: Vec<*mut c_void>,
    ) -> global::Result<(NonNull<u8>, Layout)> {
        self.unmarshal_non_purus_configuration(cfg)
            .map(|cfg| self.non_purus_call(&cfg, f_ptr, args))
    }
    pub fn non_purus_call(
        &self,
        cfg: &NonPurusCallConfiguration,
        f_ptr: *const u8,
        mut args: Vec<*mut c_void>,
    ) -> (NonNull<u8>, Layout) {
        use libffi::middle::Cif;
        fn non_purus_type_to_libffi_type(ty: &NonPurusCallType) -> libffi::middle::Type {
            use libffi::middle::Type as LibffiType;
            match ty {
                NonPurusCallType::Void => LibffiType::void(),
                NonPurusCallType::U8 => LibffiType::u8(),
                NonPurusCallType::I8 => LibffiType::i8(),
                NonPurusCallType::U16 => LibffiType::u16(),
                NonPurusCallType::I16 => LibffiType::i16(),
                NonPurusCallType::U32 => LibffiType::u32(),
                NonPurusCallType::I32 => LibffiType::i32(),
                NonPurusCallType::U64 => LibffiType::u64(),
                NonPurusCallType::I64 => LibffiType::i64(),
                NonPurusCallType::String => LibffiType::pointer(),
                NonPurusCallType::Object => LibffiType::pointer(),
                NonPurusCallType::Structure(types) => {
                    LibffiType::structure(types.iter().map(non_purus_type_to_libffi_type))
                }
            }
        }
        fn non_purus_type_arg_to_libffi_type(
            (is_by_ref, ty): &(bool, NonPurusCallType),
        ) -> libffi::middle::Type {
            use libffi::middle::Type as LibffiType;
            if *is_by_ref {
                return LibffiType::pointer();
            }
            match ty {
                NonPurusCallType::Void => LibffiType::void(),
                NonPurusCallType::U8 => LibffiType::u8(),
                NonPurusCallType::I8 => LibffiType::i8(),
                NonPurusCallType::U16 => LibffiType::u16(),
                NonPurusCallType::I16 => LibffiType::i16(),
                NonPurusCallType::U32 => LibffiType::u32(),
                NonPurusCallType::I32 => LibffiType::i32(),
                NonPurusCallType::U64 => LibffiType::u64(),
                NonPurusCallType::I64 => LibffiType::i64(),
                NonPurusCallType::String => LibffiType::pointer(),
                NonPurusCallType::Object => LibffiType::pointer(),
                NonPurusCallType::Structure(types) => {
                    LibffiType::structure(types.iter().map(non_purus_type_to_libffi_type))
                }
            }
        }
        let abi = crate::libffi_utils::get_abi_by_call_convention(cfg.call_convention);
        let cif = match cfg.call_convention {
            CallConvention::CDeclWithVararg => {
                let args = cfg.arguments.iter().map(non_purus_type_arg_to_libffi_type);
                let fixed_args = args.len() - 1;
                Cif::new_variadic_with_abi(
                    args,
                    fixed_args,
                    non_purus_type_to_libffi_type(&cfg.return_type),
                    abi,
                )
            }
            _ => Cif::new_with_abi(
                cfg.arguments.iter().map(non_purus_type_arg_to_libffi_type),
                non_purus_type_to_libffi_type(&cfg.return_type),
                abi,
            ),
        };
        let mut should_drop_pointers: DropGuard<Vec<(NonNull<c_void>, Layout)>, _> =
            DropGuard::new(Vec::new(), |x| {
                for (a, b) in x {
                    unsafe {
                        std::alloc::Allocator::deallocate(&std::alloc::Global, a.cast(), b);
                    }
                }
            });
        let mut treat_string_as_object = false;
        match cfg.encoding {
            global::non_purus_call_configuration::StringEncoding::Utf16 => todo!(),
            global::non_purus_call_configuration::StringEncoding::Utf8 => todo!(),
            global::non_purus_call_configuration::StringEncoding::C_Utf16 => {
                for (i, (is_by_ref, ty)) in cfg.arguments.iter().enumerate() {
                    if *is_by_ref {
                        continue;
                    }
                    if *ty == NonPurusCallType::String {
                        let ptr = std::alloc::Allocator::allocate(
                            &std::alloc::Global,
                            Layout::new::<*const u16>(),
                        )
                        .unwrap()
                        .as_non_null_ptr();
                        unsafe {
                            ptr.cast::<*const u16>().write(
                                args[i]
                                    .cast::<ManagedReference<Class>>()
                                    .as_ref_unchecked()
                                    .data_ptr()
                                    .cast_const()
                                    .cast(),
                            )
                        }
                        args[i] = ptr.cast().as_ptr();
                        should_drop_pointers.push((ptr.cast(), Layout::new::<*const u16>()));
                    }
                }
            }
            global::non_purus_call_configuration::StringEncoding::C_Utf8 => todo!(),
            global::non_purus_call_configuration::StringEncoding::Remain => {
                treat_string_as_object = true;
            }
        }
        match cfg.object_strategy {
            global::non_purus_call_configuration::ObjectStrategy::Remain => (),
            global::non_purus_call_configuration::ObjectStrategy::PointToData => {
                for (i, (is_by_ref, ty)) in cfg.arguments.iter().enumerate() {
                    if *is_by_ref {
                        continue;
                    }
                    if (*ty == NonPurusCallType::Object)
                        || ((*ty == NonPurusCallType::String) && treat_string_as_object)
                    {
                        let ptr = std::alloc::Allocator::allocate(
                            &std::alloc::Global,
                            Layout::new::<*mut u8>(),
                        )
                        .unwrap()
                        .as_non_null_ptr();
                        unsafe {
                            ptr.cast::<*mut u8>().write(
                                args[i]
                                    .cast::<ManagedReference<Class>>()
                                    .read()
                                    .data_ptr()
                                    .cast(),
                            )
                        }
                        should_drop_pointers.push((ptr.cast(), Layout::new::<*mut u8>()));
                    }
                }
            }
        }
        for (i, (is_by_ref, _)) in cfg.arguments.iter().enumerate() {
            if *is_by_ref {
                let p = Box::into_non_null(Box::new(args[i]));
                should_drop_pointers.push((p.cast(), Layout::new::<*mut c_void>()));
                args[i] = p.as_ptr().cast();
            }
        }
        let fun = libffi::low::CodePtr::from_ptr(f_ptr.cast());
        let result_layout = crate::memory::get_return_layout_for_libffi(cfg.return_type.layout());
        let result_ptr =
            std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, result_layout).unwrap();

        unsafe {
            libffi::raw::ffi_call(
                cif.as_raw_ptr(),
                Some(*fun.as_safe_fun()),
                result_ptr.as_non_null_ptr().cast::<c_void>().as_ptr(),
                args.as_mut_ptr(),
            );
        }

        (result_ptr.as_non_null_ptr(), result_layout)
    }
}

impl CPU {
    fn default_exception_handler<T>(&self, _method: &Method<T>) {
        use crate::value::managed_reference::StringAccessor;
        use stdlib_header::definitions::System_Exception_FieldId;
        let exception = self.get_exception().unwrap();
        if exception.is_null() {
            return;
        }

        let message = exception
            .const_access::<FieldAccessor<_>>()
            .typed_field::<ManagedReference<Class>>(
                System_Exception_FieldId::Message as _,
                Default::default(),
            )
            .unwrap();

        let stack_trace = exception
            .const_access::<FieldAccessor<_>>()
            .typed_field::<ManagedReference<Class>>(
                System_Exception_FieldId::StackTrace as _,
                Default::default(),
            )
            .unwrap();

        let mut string_builder = String::new();
        string_builder.push_str("Uncaught exception occurred: ");
        unsafe {
            string_builder.as_mut_vec().append(
                &mut message
                    .access::<StringAccessor>()
                    .unwrap()
                    .to_string_lossy()
                    .unwrap()
                    .into_bytes(),
            );
        }
        for stack_trace in unsafe {
            stack_trace
                .access::<ArrayAccessor>()
                .unwrap()
                .as_slice::<ManagedReference<Class>>()
                .unwrap()
        } {
            <_ as std::fmt::Write>::write_fmt(
                &mut string_builder,
                format_args!(
                    "{}\tat {}",
                    LineEnding::default(),
                    stack_trace
                        .access::<StringAccessor>()
                        .unwrap()
                        .to_string_lossy()
                        .unwrap()
                ),
            )
            .unwrap();
        }
        string_builder.push_str(LineEnding::default().as_str());
        let mut stderr = std::io::stderr().lock();
        <_ as std::io::Write>::write_all(&mut stderr, string_builder.as_bytes()).unwrap();
        <_ as std::io::Write>::flush(&mut stderr).unwrap();
    }

    pub fn invoke_main<T: GetAssemblyRef + GetTypeVars + GetNonGenericTypeHandleKind>(
        &self,
        method: &Method<T>,
        args: Vec<String>,
    ) -> MainResult {
        if !method.attr().is_static() {
            dt_println!("Main should be static");
            return MainResult::SignatureNotMatched;
        }

        enum ResultKind {
            UInt8,
            Void,
        }
        let result_kind = match method.get_return_type().get_core_type_id() {
            Some(CoreTypeId::System_Void) => ResultKind::Void,
            Some(CoreTypeId::System_UInt8) => ResultKind::UInt8,
            _ => {
                dt_println!("Main's return type should be either System.UInt8 or System.Void");
                return MainResult::SignatureNotMatched;
            }
        };

        match &**method.args() {
            [] => match result_kind {
                ResultKind::UInt8 => {
                    let res = method.typed_res_call::<u8>(self, None, &[]);
                    if self.has_exception().unwrap() {
                        self.default_exception_handler(method);
                        MainResult::Custom(res)
                    } else {
                        MainResult::Custom(res)
                    }
                }
                ResultKind::Void => {
                    method.typed_res_call::<()>(self, None, &[]);
                    if self.has_exception().unwrap() {
                        self.default_exception_handler(method);
                        MainResult::VoidWithException
                    } else {
                        MainResult::Void
                    }
                }
            },
            [args_param] => {
                let ty = args_param.get_type(method);
                if ty
                    .get_core_type_id()
                    .is_none_or(|x| x != CoreTypeId::System_Array_1)
                {
                    dt_println!(
                        "Main's first argument should be of type System::Array`1[System::String]"
                    );
                    return MainResult::SignatureNotMatched;
                }
                let ty = ty.unwrap_class();
                let string_t = self
                    .vm_ref()
                    .assembly_manager()
                    .get_core_type(CoreTypeId::System_String)
                    .unwrap_class();

                unsafe {
                    if !ty.as_ref().type_vars().as_deref().is_some_and(|x| {
                        x.get(0).is_some_and(|x| {
                            x.load(self.vm_ref().assembly_manager())
                                .map(|x| x.get_non_generic_with_type(ty.as_ref()))
                                .is_some_and(|x| x.is_certain_core_type(CoreTypeId::System_Array_1))
                        })
                    }) {
                        dt_println!(
                            "Main's first argument should be of type System::Array`1[System::String]"
                        );
                        return MainResult::SignatureNotMatched;
                    }
                }

                let mut arg1_obj = ManagedReference::alloc_array(
                    self,
                    unsafe { *string_t.as_ref().method_table() },
                    args.len(),
                );

                unsafe {
                    for (arg_pl, arg_rs) in arg1_obj
                        .access_unchecked_mut::<ArrayAccessor>()
                        .as_slice_mut::<ManagedReference<Class>>()
                        .unwrap()
                        .iter_mut()
                        .zip(args)
                    {
                        *arg_pl = ManagedReference::new_string(self, &arg_rs);
                    }
                }

                let result = match result_kind {
                    ResultKind::UInt8 => {
                        let res = method.typed_res_call::<u8>(
                            self,
                            None,
                            &[(&raw const arg1_obj).cast_mut().cast()],
                        );
                        if self.has_exception().unwrap() {
                            self.default_exception_handler(method);
                            MainResult::Custom(res)
                        } else {
                            MainResult::Custom(res)
                        }
                    }
                    ResultKind::Void => {
                        method.typed_res_call::<()>(
                            self,
                            None,
                            &[(&raw const arg1_obj).cast_mut().cast()],
                        );
                        if self.has_exception().unwrap() {
                            self.default_exception_handler(method);
                            MainResult::VoidWithException
                        } else {
                            MainResult::Void
                        }
                    }
                };
                arg1_obj.destroy(self);

                result
            }
            _ => {
                dt_println!("Main's arguments should be less than 2");
                MainResult::SignatureNotMatched
            }
        }
    }

    pub fn invoke_main_and_exit<T: GetAssemblyRef + GetTypeVars + GetNonGenericTypeHandleKind>(
        &self,
        method: &Method<T>,
        args: Vec<String>,
    ) -> ! {
        let result = self.invoke_main(method, args);
        result.report().exit_process()
    }
}

impl CPU {
    pub fn new_object(
        &self,
        ty: &MaybeUnloadedTypeHandle,
        ctor_name: &MethodRef,
        args: &[*mut c_void],
    ) -> Option<ManagedReference<Class>> {
        let Some(TypeHandle::Class(class)) = ty.load(self.vm_ref().assembly_manager()) else {
            return None;
        };

        let class_ref = unsafe { class.as_ref() };
        let mt = class_ref.method_table_ref();

        let obj = ManagedReference::<Class>::common_alloc(self, NonNull::from_ref(mt), false);

        let Some(ctor) = mt.get_method_by_ref(ctor_name) else {
            return None;
        };
        unsafe {
            ctor.as_ref()
                .typed_res_call::<()>(self, Some(NonNull::from_ref(&obj).cast()), &args);
        }
        Some(obj)
    }
}
