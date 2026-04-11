use std::{
    alloc::{Allocator as _, Layout},
    ffi::c_void,
    ptr::NonNull,
};

use global::{
    instruction::{IRegisterAddr, Instruction, RegisterAddr},
    t_println,
};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        type_handle::MaybeUnloadedTypeHandle,
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
    ) -> Option<Result<(), Termination>> {
        let Some(ins) = method.instructions.get(*pc) else {
            return Some(Err(Termination::AllInstructionExecuted));
        };

        if !matches!(ins, Instruction::Nop) {
            #[cfg(feature = "print_invoke_and_call")]
            eprintln!("INVOKE: {ins}");
        }

        macro _eval($ins:ident by $evaluator:ident) {
            $evaluator::eval(method, cpu, this, args, result_ptr, pc, $ins)
        }

        match ins {
            Instruction::Nop => Some(Ok(())),
            Instruction::Load(ins) => _eval!(ins by load),
            Instruction::SLoad(ins) => _eval!(ins by load),

            Instruction::ReadPointerTo(ins) => {
                read_write_pointer::read_pointer_to(method, cpu, this, args, result_ptr, pc, ins)
            }
            Instruction::SReadPointerTo(ins) => {
                read_write_pointer::read_pointer_to(method, cpu, this, args, result_ptr, pc, ins)
            }

            Instruction::WritePointer(ins) => {
                read_write_pointer::write_pointer(method, cpu, this, args, result_ptr, pc, ins)
            }
            Instruction::SWritePointer(ins) => {
                read_write_pointer::write_pointer(method, cpu, this, args, result_ptr, pc, ins)
            }

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

            Instruction::Throw { exception_addr } => {
                eval_throw(method, cpu, this, args, result_ptr, pc, exception_addr)
            }
            Instruction::SThrow { exception_addr } => {
                eval_throw(method, cpu, this, args, result_ptr, pc, exception_addr)
            }

            Instruction::ReturnVal { register_addr } => {
                eval_return_val(method, cpu, this, args, result_ptr, pc, register_addr)
            }
            Instruction::SReturnVal { register_addr } => {
                eval_return_val(method, cpu, this, args, result_ptr, pc, register_addr)
            }

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
    ) -> Result<(), Termination> {
        if let Some(res) = Self::common_match_code(method, cpu, this, args, result_ptr, pc) {
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

    loop {
        if cpu.has_exception() {
            return (result_ptr.cast(), result_layout);
        }
        if let Err(t) = T::spec_match_code(method, cpu, this, args, result_ptr, &mut pc) {
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
                Termination::LoadTypeHandleFailed(_) => {
                    t_println!("Cannot load TypeHandle");
                }
                Termination::LoadMethodFailed(_) => {
                    t_println!("Cannot load Method");
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

                Termination::Terminated => {}
                Termination::Returned => {}
            }
            break (result_ptr.cast(), result_layout);
        }
        pc += 1;
    }
}
