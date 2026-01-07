use std::{
    ptr::{NonNull, Unique},
    sync::{LockResult, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    stdlib::CoreTypeId,
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetNonGenericTypeHandleKind, GetTypeVars},
        method::Method,
    },
    value::managed_reference::{ArrayAccessor, ManagedReference},
};

use super::VirtualMachine;

mod call_stack;
mod exception;
mod mem_record;

pub use call_stack::{CallStack, CallStackFrame, CommonCallStackFrame, NativeCallStackFrame};
use enumflags2::BitFlags;
pub use exception::ExceptionManager;
use global::dt_println;
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
            Self::SignatureNotMatched => std::process::ExitCode::FAILURE,
        }
    }
}

impl CPU {
    fn default_exception_handler<T>(&self, method: &Method<T>) {
        use crate::stdlib::System_Exception_FieldId;
        use crate::value::managed_reference::StringAccessor;
        let exception = self.get_exception().unwrap();
        if exception.is_null() {
            return;
        }

        let message = exception
            .typed_field::<ManagedReference<Class>>(
                false,
                System_Exception_FieldId::Message as _,
                Default::default(),
            )
            .unwrap();

        let stack_trace = exception
            .typed_field::<ManagedReference<Class>>(
                false,
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
            unsafe {
                string_builder.as_mut_vec().append(
                    &mut format!(
                        "{}\tat {}",
                        LineEnding::default(),
                        stack_trace
                            .access::<StringAccessor>()
                            .unwrap()
                            .to_string_lossy()
                            .unwrap()
                    )
                    .into_bytes(),
                )
            }
        }
        <_ as std::fmt::Write>::write_fmt(
            &mut string_builder,
            format_args!(
                "{}by {}{0}",
                LineEnding::default(),
                method.display(BitFlags::all())
            ),
        )
        .unwrap();

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
                        "Main's first argument should be of type System.Array`1[System.String]"
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
                            "Main's first argument should be of type System.Array`1[System.String]"
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
                        *arg_pl = ManagedReference::new_string(self, arg_rs);
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
}
