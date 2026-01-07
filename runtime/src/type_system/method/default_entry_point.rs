use std::{
    alloc::{Allocator as _, Layout},
    ffi::c_void,
    fmt::Write as _,
    mem::MaybeUninit,
    ptr::NonNull,
};

use enumflags2::BitFlags;
use global::{instruction::Instruction, t_println};
use line_ending::LineEnding;

use crate::{
    stdlib::{CoreTypeId, System_Exception_FieldId},
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    value::managed_reference::{ArrayAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

use super::{Method, MethodRef};

#[allow(dead_code)]
enum Termination {
    LoadRegisterFailed(u64),
    LoadArgFailed(u64),
    NullReference,
    AllInstructionExecuted,
    LoadTypeHandleFailed(MaybeUnloadedTypeHandle),
    LoadMethodFailed(MethodRef),
    LoadFieldFailed(u32),

    Returned,
    Terminated,
}

trait Spec: Sized + GetAssemblyRef + GetTypeVars {
    /// Return false if it's terminated
    fn spec_match_code(
        method: &Method<Self>,
        cpu: &CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        result_ptr: NonNull<[u8]>,
        pc: &mut usize,
    ) -> Result<(), Termination>;

    /// Return None if the current instruction cannot be handled through common way,
    /// false if it's terminated
    fn common_match_code(
        method: &Method<Self>,
        cpu: &CPU,
        this: Option<NonNull<()>>,
        args: &[*mut c_void],
        result_ptr: NonNull<[u8]>,
        pc: &mut usize,
    ) -> Option<Result<(), Termination>> {
        let Some(ins) = method.instructions.get(*pc) else {
            return Some(Err(Termination::AllInstructionExecuted));
        };
        dbg!(ins);

        let frame = cpu.current_common_call_frame().unwrap().unwrap();

        match ins {
            Instruction::LoadTrue { register_addr } => {
                if !frame.write_typed(*register_addr, true) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }
            Instruction::LoadFalse { register_addr } => {
                if !frame.write_typed(*register_addr, false) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }
            Instruction::Load_u8 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }
            Instruction::Load_u64 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }
            Instruction::LoadThis { register_addr } => {
                let Some((register_ptr, register_layout)) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                debug_assert!(register_layout.size() >= Layout::new::<*const ()>().size());
                let Some(this) = this else {
                    return Some(Err(Termination::NullReference));
                };
                unsafe {
                    register_ptr.copy_from(this.cast(), Layout::new::<*const ()>().size());
                }
            }
            Instruction::Load_String { register_addr, val } => {
                let val_obj = ManagedReference::new_string(cpu, val.as_str().to_owned());
                if !frame.write_typed(*register_addr, val_obj) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }

            Instruction::NewObject {
                ty,
                ctor_name,
                args,
                register_addr,
            } => {
                let Some(NonGenericTypeHandle::Class(class)) = ty
                    .load(cpu.vm_ref().assembly_manager())
                    .map(|x| x.get_non_generic_with_method(method))
                else {
                    return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
                };

                drop(frame);

                let class_ref = unsafe { class.as_ref() };
                let mt = class_ref.method_table_ref();

                let obj =
                    ManagedReference::<Class>::common_alloc(cpu, NonNull::from_ref(mt), false);

                let frame = cpu.current_common_call_frame().unwrap().unwrap();
                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().0.cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();
                drop(frame);

                let Some(ctor) = mt.get_method_by_ref(ctor_name) else {
                    return Some(Err(Termination::LoadMethodFailed(ctor_name.clone())));
                };
                unsafe {
                    ctor.as_ref().typed_res_call::<()>(
                        cpu,
                        Some(NonNull::from_ref(&obj).cast()),
                        &args,
                    );
                }

                let frame = cpu.current_common_call_frame().unwrap().unwrap();
                if !frame.write_typed(*register_addr, obj) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                }
            }
            Instruction::InstanceCall {
                val,
                method,
                args,
                ret_at,
            } => {
                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().0.cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();

                let Some(val) = frame.get_typed::<ManagedReference<Class>>(*val).copied() else {
                    return Some(Err(Termination::LoadRegisterFailed(*val)));
                };
                if val.is_null() {
                    return Some(Err(Termination::NullReference));
                }
                drop(frame);
                let mt = val.method_table_ref().unwrap();
                let Some(m) = mt.get_method_by_ref(method) else {
                    return Some(Err(Termination::LoadMethodFailed(method.clone())));
                };

                let m_ref = unsafe { m.as_ref() };
                let actual_layout = m_ref.get_return_type().val_layout();
                let (ret_ptr, ret_layout) =
                    m_ref.untyped_call(cpu, Some(NonNull::from_ref(&val).cast()), &args);

                if actual_layout != Layout::new::<()>() {
                    let frame = cpu.current_common_call_frame().unwrap().unwrap();
                    let Some((out_ptr, out_layout)) = frame.get(*ret_at) else {
                        return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                    };
                    debug_assert!(out_layout.size() >= ret_layout.size());
                    unsafe {
                        out_ptr.copy_from(ret_ptr, ret_layout.size());
                    }
                }
                unsafe {
                    std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
                }
            }
            Instruction::StaticCall {
                ty,
                method: m_target,
                args,
                ret_at,
            } => {
                let Some(ty) = ty
                    .load(cpu.vm_ref().assembly_manager())
                    .map(|x| x.get_non_generic_with_method(method))
                else {
                    return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
                };

                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().0.cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();

                macro apply($n:ident $b:block) {
                    match ty {
                        NonGenericTypeHandle::Class($n) => $b,
                        NonGenericTypeHandle::Struct($n) => $b,
                    }
                }

                let actual_layout;

                let (ret_ptr, ret_layout) = apply! {
                    t {
                        let t_ref = unsafe { t.as_ref() };
                        let mt = t_ref.method_table_ref();
                        let m = mt.get_method_by_ref(m_target).unwrap();
                        let m_ref = unsafe { m.as_ref() };
                        drop(frame);
                        actual_layout = m_ref.get_return_type().val_layout();

                        m_ref.untyped_call(cpu, None, &args)
                    }
                };

                if actual_layout != Layout::new::<()>() {
                    let frame = cpu.current_common_call_frame().unwrap().unwrap();
                    let Some((out_ptr, out_layout)) = frame.get(*ret_at) else {
                        return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                    };
                    debug_assert!(out_layout.size() >= actual_layout.size());
                    unsafe {
                        out_ptr.copy_from(ret_ptr, ret_layout.size());
                    }
                }
                unsafe {
                    std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
                }
            }
            Instruction::LoadArg { register_addr, arg } => {
                let Some(arg) = args.get((*arg) as usize) else {
                    return Some(Err(Termination::LoadArgFailed(*arg)));
                };

                match NonNull::new(*arg) {
                    None => {
                        frame.zero_register(*register_addr);
                    }
                    Some(p) => {
                        let Some((out_p, out_l)) = frame.get(*register_addr) else {
                            return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                        };
                        unsafe { out_p.copy_from(p.cast(), out_l.size()) }
                    }
                }
            }
            Instruction::LoadStatic {
                register_addr,
                ty,
                field,
            } => {
                let Some(ty) = ty
                    .load(cpu.vm_ref().assembly_manager())
                    .map(|x| x.get_non_generic_with_method(method))
                else {
                    return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
                };
                drop(frame);
                let Some((f_ptr, f_layout)) = cpu.vm_ref().get_static_field(ty, *field) else {
                    return Some(Err(Termination::LoadFieldFailed(*field)));
                };
                let frame = cpu.current_common_call_frame().unwrap().unwrap();
                let Some((out_ptr, out_layout)) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                debug_assert!(out_layout.size() >= f_layout.size());
                unsafe {
                    out_ptr.copy_from(f_ptr.cast(), f_layout.size());
                }
            }
            Instruction::SetThisField { val_addr, field } => {
                let Some((val_p, val_layout)) = frame.get(*val_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*val_addr)));
                };
                let Some(this) = this else {
                    return Some(Err(Termination::NullReference));
                };
                unsafe {
                    let this = this.cast::<ManagedReference<Class>>().as_ref();
                    let Some((f_ptr, f_layout)) = this.field(false, *field, Default::default())
                    else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    debug_assert!(val_layout.size() >= f_layout.size());
                    f_ptr.cast::<u8>().copy_from(val_p, f_layout.size());
                }
            }
            Instruction::SetStaticField {
                val_addr,
                ty,
                field,
            } => {
                let Some(ty) = ty
                    .load(cpu.vm_ref().assembly_manager())
                    .map(|x| x.get_non_generic_with_method(method))
                else {
                    return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
                };
                drop(frame);
                let Some((f_ptr, f_layout)) = cpu.vm_ref().get_static_field(ty, *field) else {
                    return Some(Err(Termination::LoadFieldFailed(*field)));
                };
                let frame = cpu.current_common_call_frame().unwrap().unwrap();
                let Some((val_ptr, val_layout)) = frame.get(*val_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*val_addr)));
                };
                debug_assert!(val_layout.size() <= f_layout.size());
                unsafe {
                    val_ptr.copy_to(f_ptr.cast(), val_layout.size());
                }
            }
            Instruction::Throw { exception_addr } => {
                let Some(exception) = frame.get_typed::<ManagedReference<Class>>(*exception_addr)
                else {
                    return Some(Err(Termination::LoadRegisterFailed(*exception_addr)));
                };
                assert!(!exception.is_null());
                cpu.throw_exception(*exception).unwrap();
            }
            Instruction::ReturnVal { register_addr } => {
                let Some((res_ptr, res_layout)) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                debug_assert!(result_ptr.len() >= res_layout.size());
                unsafe {
                    result_ptr
                        .as_non_null_ptr()
                        .copy_from(res_ptr, res_layout.size());
                }
            }
        }

        Some(Ok(()))
    }
}

impl<T: GetAssemblyRef + GetTypeVars> Spec for T {
    default fn spec_match_code(
        method: &Method<Self>,
        cpu: &CPU,
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
    cpu: &CPU,
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
        if let Err(t) = T::spec_match_code(method, cpu, this, args, result_ptr, &mut pc) {
            match t {
                Termination::LoadRegisterFailed(r) => {
                    t_println!("Cannot load Register {r}");
                }
                Termination::LoadArgFailed(a) => {
                    t_println!("Cannot load Arg {a}");
                }
                Termination::NullReference => {
                    t_println!("NULL PTR");
                }
                Termination::AllInstructionExecuted => unsafe {
                    result_ptr.as_uninit_slice_mut().fill(MaybeUninit::zeroed());
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

                Termination::Terminated => (),
                Termination::Returned => {}
            }
            break (result_ptr.cast(), result_layout);
        }
        pc += 1;
        if cpu.has_exception().unwrap() {
            return (result_ptr.cast(), result_layout);
        }
    }
}

pub fn default_exception_handler<T>(cpu: &CPU, method: &Method<T>) {
    let exception = cpu.get_exception().unwrap();
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
    write!(
        &mut string_builder,
        "{}by {}{0}",
        LineEnding::default(),
        method.display(BitFlags::all())
    )
    .unwrap();

    let mut stderr = std::io::stderr().lock();
    <_ as std::io::Write>::write_all(&mut stderr, string_builder.as_bytes()).unwrap();
    <_ as std::io::Write>::flush(&mut stderr).unwrap();
}
