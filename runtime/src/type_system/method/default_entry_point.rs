use std::{
    alloc::{Allocator as _, Layout},
    ffi::c_void,
    ptr::NonNull,
    range::Range,
};

use global::{
    instruction::{IRegisterAddr, Instruction, RegisterAddr},
    t_println,
};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        r#struct::Struct,
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandleKind},
    },
    value::managed_reference::ManagedReference,
    virtual_machine::cpu::{CPU, CommonCallStackFrame},
};

use super::{Method, MethodRef};

#[allow(dead_code)]
enum Termination {
    LoadRegisterFailed(RegisterAddr),
    LoadArgFailed(u64),
    NullReference(&'static core::panic::Location<'static>),
    AllInstructionExecuted,
    LoadTypeHandleFailed(MaybeUnloadedTypeHandle),
    LoadMethodFailed(MethodRef),
    LoadFieldFailed(u32),
    NewObjectFailed,
    /// Failed to convert a puralingua object to rust
    UnmarshalFailed(global::Error),
    UnimplementedInterface,
    RethrowWithoutExceptions,
    LoadCaughtExceptionWithoutExceptions,

    Returned,
    Terminated,
}

macro load_register_failed($addr:expr) {
    return Some(Err(Termination::LoadRegisterFailed($addr.into_generic())))
}

fn call_frame(cpu: &CPU) -> &CommonCallStackFrame {
    cpu.current_common_call_frame().unwrap()
}

mod load;

mod read_write_pointer;

mod check;

mod new;

mod call;

mod set;

mod jump;

mod calculate;

fn eval_throw<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    #[allow(unused)] caught_exception: Option<ManagedReference<Class>>,
    exception_addr: &TRegisterAddr,
) -> Option<Result<(), Termination>> {
    let Some(exception) = call_frame(cpu).get_typed::<ManagedReference<Class>, _>(*exception_addr)
    else {
        load_register_failed!(*exception_addr);
    };
    assert!(!exception.is_null());
    cpu.throw_exception(*exception);
    Some(Ok(()))
}

fn eval_return_val<T: Sized + GetAssemblyRef + GetTypeVars, TRegisterAddr: IRegisterAddr>(
    #[allow(unused)] method: &Method<T>,
    #[allow(unused)] cpu: &mut CPU,
    #[allow(unused)] this: Option<NonNull<()>>,
    #[allow(unused)] args: &[*mut c_void],
    #[allow(unused)] result_ptr: NonNull<[u8]>,
    #[allow(unused)] pc: &mut usize,
    #[allow(unused)] caught_exception: Option<ManagedReference<Class>>,
    register_addr: &TRegisterAddr,
) -> Option<Result<(), Termination>> {
    let Some(res_var) = call_frame(cpu).get(*register_addr) else {
        load_register_failed!(*register_addr);
    };
    unsafe {
        res_var.copy_all_to(result_ptr.as_non_null_ptr());
    }
    return Some(Err(Termination::Returned));
}

trait Spec: Sized + GetAssemblyRef + GetTypeVars {
    /// Return false if it's terminated
    fn spec_match_code(
        method: &Method<Self>,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        result_ptr: NonNull<[u8]>,
        pc: &mut usize,
        caught_exception: Option<ManagedReference<Class>>,
    ) -> Result<(), Termination>;

    /// Return None if the current instruction cannot be handled through common way,
    /// false if it's terminated
    fn common_match_code(
        method: &Method<Self>,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        result_ptr: NonNull<[u8]>,
        pc: &mut usize,
        caught_exception: Option<ManagedReference<Class>>,
    ) -> Option<Result<(), Termination>> {
        let Some(ins) = method.instructions.get(*pc) else {
            return Some(Err(Termination::AllInstructionExecuted));
        };

        if !matches!(ins, Instruction::Nop) {
            #[cfg(feature = "print_invoke_and_call")]
            eprintln!("INVOKE: {ins}");
        }

        macro _eval($ins:ident by $evaluator:ident) {
            $evaluator::eval(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                $ins,
            )
        }

        match ins {
            Instruction::Nop => Some(Ok(())),
            Instruction::Load(ins) => _eval!(ins by load),
            Instruction::SLoad(ins) => _eval!(ins by load),

            Instruction::ReadPointerTo(ins) => read_write_pointer::read_pointer_to(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                ins,
            ),
            Instruction::SReadPointerTo(ins) => read_write_pointer::read_pointer_to(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                ins,
            ),

            Instruction::WritePointer(ins) => read_write_pointer::write_pointer(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                ins,
            ),
            Instruction::SWritePointer(ins) => read_write_pointer::write_pointer(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                ins,
            ),

            Instruction::Check(ins) => _eval!(ins by check),
            Instruction::SCheck(ins) => _eval!(ins by check),

            Instruction::New(ins) => _eval!(ins by new),
            Instruction::SNew(ins) => _eval!(ins by new),

            Instruction::Call(ins) => _eval!(ins by call),
            Instruction::SCall(ins) => _eval!(ins by call),

            Instruction::Set(ins) => _eval!(ins by set),
            Instruction::SSet(ins) => _eval!(ins by set),

            Instruction::Calculate(ins) => _eval!(ins by calculate),
            Instruction::SCalculate(ins) => _eval!(ins by calculate),

            Instruction::Throw { exception_addr } => eval_throw(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                exception_addr,
            ),
            Instruction::SThrow { exception_addr } => eval_throw(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                exception_addr,
            ),
            Instruction::Rethrow => {
                let Some(exception) = caught_exception else {
                    return Some(Err(Termination::RethrowWithoutExceptions));
                };
                cpu.throw_exception(exception);
                Some(Ok(()))
            }

            Instruction::ReturnVal { register_addr } => eval_return_val(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                register_addr,
            ),
            Instruction::SReturnVal { register_addr } => eval_return_val(
                method,
                cpu,
                this,
                args,
                result_ptr,
                pc,
                caught_exception,
                register_addr,
            ),

            Instruction::Jump(ins) => _eval!(ins by jump),
            Instruction::SJump(ins) => _eval!(ins by jump),
        }
    }
}

impl<T: GetAssemblyRef + GetTypeVars> Spec for T {
    default fn spec_match_code(
        method: &Method<Self>,
        cpu: &mut CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        result_ptr: NonNull<[u8]>,
        pc: &mut usize,
        caught_exception: Option<ManagedReference<Class>>,
    ) -> Result<(), Termination> {
        if let Some(res) =
            Self::common_match_code(method, cpu, this, args, result_ptr, pc, caught_exception)
        {
            return res;
        }
        Ok(())
    }
}

#[allow(improper_ctypes_definitions)] // It is always called in rust.
pub extern "system" fn __default_entry_point<T: GetTypeVars + GetAssemblyRef>(
    method: &Method<T>,
    cpu: &mut CPU,
    this: Option<NonNull<()>>,
    args: &[*mut c_void],
) -> (NonNull<u8>, Layout) {
    let mut result_layout = method.get_return_type().val_layout();
    if result_layout.size() < size_of::<usize>() {
        result_layout = Layout::new::<usize>();
    }
    let result_ptr = std::alloc::Global.allocate(result_layout).unwrap();
    let mut pc = 0;
    let mut caught_exception = vec![];

    enum RunStatus {
        OnFinally {
            end: u64,
            recover: usize,
            then_fault: Option<Range<u64>>,
            then_catch: Range<u64>,
        },
        OnFault {
            end: u64,
            recover: usize,
            then_catch: Range<u64>,
        },
        OnCatch {
            end: u64,
            recover: usize,
        },
    }

    let mut status = vec![];

    loop {
        if let Some(last_status) = status.pop() {
            match last_status {
                RunStatus::OnFinally {
                    end,
                    recover,
                    then_fault,
                    then_catch,
                } => {
                    if (pc as u64) >= end {
                        if let Some(fault_range) = then_fault {
                            status.push(RunStatus::OnFault {
                                end: fault_range.end,
                                recover,
                                then_catch: then_catch,
                            });
                            pc = fault_range.start as usize;
                        } else {
                            status.push(RunStatus::OnCatch {
                                end: then_catch.end,
                                recover,
                            });
                            pc = then_catch.start as usize;
                        }

                        continue;
                    }
                }
                RunStatus::OnFault {
                    end,
                    recover,
                    then_catch,
                } => {
                    if (pc as u64) >= end {
                        status.push(RunStatus::OnCatch {
                            end: then_catch.end,
                            recover,
                        });
                        pc = then_catch.start as usize;
                        continue;
                    }
                }
                RunStatus::OnCatch { end, recover } => {
                    if (pc as u64) >= end {
                        pc = recover;
                        continue;
                    }
                }
            }
        }

        if cpu.has_exception() {
            if let Some(handler) = method.exception_table.get_for(pc).find_map(|x| {
                x.get_exception_type(method)
                    .filter(|exception_type| {
                        cpu.is_exception_type_suitable(*exception_type) && {
                            x.get_filter(method).is_none_or(
                                |(ty_kind, filter_method)| match ty_kind {
                                    NonGenericTypeHandleKind::Class => {
                                        let filter_method = filter_method.cast::<Method<Class>>();
                                        let enable = unsafe { filter_method.as_ref() }
                                            .typed_res_call::<u8>(cpu, None, &[]);
                                        enable != 0
                                    }
                                    NonGenericTypeHandleKind::Struct => {
                                        let filter_method = filter_method.cast::<Method<Struct>>();
                                        let enable = unsafe { filter_method.as_ref() }
                                            .typed_res_call::<u8>(cpu, None, &[]);
                                        enable != 0
                                    }
                                    NonGenericTypeHandleKind::Interface => unreachable!(),
                                },
                            )
                        }
                    })
                    .map(|_| x)
            }) {
                caught_exception.push({
                    let x = cpu.take_exception();
                    debug_assert_ne!(x, ManagedReference::null());
                    x
                });
                match (handler.finally(), handler.fault(), handler.catch()) {
                    (Some(finally), fault, catch) => {
                        status.push(RunStatus::OnFinally {
                            end: finally.end,
                            recover: pc,
                            then_fault: fault,
                            then_catch: catch,
                        });
                        pc = finally.start as usize;
                        continue;
                    }
                    (None, Some(fault), catch) => {
                        status.push(RunStatus::OnFault {
                            end: fault.end,
                            recover: pc,
                            then_catch: catch,
                        });
                        pc = fault.start as usize;
                        continue;
                    }
                    (None, None, catch) => {
                        status.push(RunStatus::OnCatch {
                            end: catch.end,
                            recover: pc,
                        });
                        pc = catch.start as usize;
                        continue;
                    }
                }
            } else {
                return (result_ptr.cast(), result_layout);
            }
        }
        if let Err(t) = T::spec_match_code(
            method,
            cpu,
            this,
            args,
            result_ptr,
            &mut pc,
            caught_exception.last().copied(),
        ) {
            match t {
                Termination::LoadRegisterFailed(r) => {
                    t_println!("Cannot load Register {r}");
                }
                Termination::LoadArgFailed(a) => {
                    t_println!("Cannot load Arg {a}");
                }
                Termination::NullReference(location) => {
                    t_println!("NULL PTR at {location}");
                }
                Termination::AllInstructionExecuted => unsafe {
                    #[cfg(debug_assertions)]
                    {
                        if !method
                            .get_return_type()
                            .is_certain_core_type(stdlib_header::CoreTypeId::System_Void)
                        {
                            panic!("Not-return-void method returns nothing");
                        }

                        result_ptr
                            .as_non_null_ptr()
                            .write_bytes(0, result_ptr.len());
                    }
                },
                Termination::LoadTypeHandleFailed(th) => {
                    t_println!("Cannot load TypeHandle {}", th);
                }
                Termination::LoadMethodFailed(m) => {
                    t_println!("Cannot load Method {}", m);
                }
                Termination::LoadFieldFailed(f) => {
                    t_println!("Cannot load Field {f}");
                }
                Termination::NewObjectFailed => {
                    t_println!("NewObject failed");
                }
                Termination::UnmarshalFailed(err) => {
                    t_println!("Unmarshal failed because:\n{err}");
                }
                Termination::UnimplementedInterface => {
                    t_println!("Interface has not been implemented");
                }
                Termination::RethrowWithoutExceptions => {
                    t_println!("Rethrow without exceptions");
                }
                Termination::LoadCaughtExceptionWithoutExceptions => {
                    t_println!("Load caught exception without exceptions");
                }

                Termination::Terminated => {}
                Termination::Returned => {}
            }
            break (result_ptr.cast(), result_layout);
        }
        pc += 1;
    }
}
