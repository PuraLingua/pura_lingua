use std::{
    alloc::{Allocator as _, Layout},
    ffi::c_void,
    ptr::NonNull,
};

use global::{
    instruction::{Instruction, JumpTarget, JumpTargetType, RegisterAddr},
    t_println,
};

use crate::{
    type_system::{
        class::Class,
        get_traits::{GetAssemblyRef, GetTypeVars},
        type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle},
    },
    value::managed_reference::{FieldAccessor, ManagedReference},
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

    Returned,
    Terminated,
}

fn do_jump(pc: &mut usize, target: JumpTarget) {
    match target.ty() {
        JumpTargetType::Absolute => {
            *pc = (target.val() as usize) - 1;
        }
        JumpTargetType::Forward => {
            <_ as std::ops::AddAssign>::add_assign(pc, (target.val() as usize) - 1);
        }
        JumpTargetType::Backward => {
            <_ as std::ops::SubAssign>::sub_assign(pc, (target.val() as usize) - 1);
        }
        JumpTargetType::Unknown => unreachable!(),
    }
}

macro load_register_failed($addr:expr) {
    return Some(Err(Termination::LoadRegisterFailed($addr)))
}

fn call_frame(cpu: &CPU) -> std::sync::MappedRwLockReadGuard<'_, CommonCallStackFrame> {
    cpu.current_common_call_frame().unwrap().unwrap()
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
        println!("{}", ins);

        let frame = call_frame(cpu);

        match ins {
            Instruction::Nop => (),
            Instruction::LoadTrue { register_addr } => {
                if !frame.write_typed(*register_addr, true) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::LoadFalse { register_addr } => {
                if !frame.write_typed(*register_addr, false) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_u8 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_u16 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_u32 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_u64 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }

            Instruction::Load_i8 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_i16 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_i32 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::Load_i64 { register_addr, val } => {
                if !frame.write_typed(*register_addr, *val) {
                    load_register_failed!(*register_addr);
                }
            }

            Instruction::LoadThis { register_addr } => {
                let Some(local_var) = frame.get(*register_addr) else {
                    load_register_failed!(*register_addr);
                };
                debug_assert!(local_var.layout.size() >= Layout::new::<*const ()>().size());
                let Some(this) = this else {
                    return Some(Err(Termination::NullReference(
                        core::panic::Location::caller(),
                    )));
                };
                unsafe {
                    local_var
                        .ptr
                        .copy_from(this.cast(), Layout::new::<*const ()>().size());
                }
            }
            Instruction::Load_String { register_addr, val } => {
                let val_obj = ManagedReference::new_string(cpu, val);
                if !frame.write_typed(*register_addr, val_obj) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::LoadTypeValueSize { register_addr, ty } => {
                let Some(ty) = ty.load(cpu.vm_ref().assembly_manager()) else {
                    return Some(Err(Termination::LoadTypeHandleFailed(ty.clone())));
                };
                let ty = ty.get_non_generic_with_method(method);
                let size = ty.val_layout().size();
                if !frame.write_typed(*register_addr, size) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::ReadPointerTo {
                ptr,
                size,
                destination,
            } => {
                let Some(&ptr_var) = frame.get_typed::<*const u8>(*ptr) else {
                    load_register_failed!(*ptr);
                };
                if ptr_var.is_null() {
                    return Some(Err(Termination::NullReference(
                        core::panic::Location::caller(),
                    )));
                }
                let Some(size) = frame.get_typed(*size) else {
                    load_register_failed!(*size);
                };
                let Some(destination) = frame.get(*destination) else {
                    load_register_failed!(*destination);
                };
                unsafe {
                    ptr_var.copy_to(destination.ptr.as_ptr(), *size);
                }
            }
            Instruction::WritePointer { source, size, ptr } => {
                let Some(source) = frame.get(*source) else {
                    load_register_failed!(*source);
                };
                let Some(size) = frame.get_typed::<usize>(*size) else {
                    load_register_failed!(*size);
                };
                let Some(&ptr_var) = frame.get_typed::<*const u8>(*ptr) else {
                    load_register_failed!(*ptr);
                };
                let Some(ptr_var) = NonNull::new(ptr_var.cast_mut()) else {
                    return Some(Err(Termination::NullReference(
                        core::panic::Location::caller(),
                    )));
                };
                unsafe {
                    source.copy_to(ptr_var, *size);
                }
            }

            Instruction::IsAllZero {
                register_addr,
                to_check,
            } => {
                let Some(to_check_var) = frame.get(*to_check) else {
                    load_register_failed!(*to_check);
                };
                let res = to_check_var.is_all_zero();
                if !frame.write_typed(*register_addr, res) {
                    load_register_failed!(*register_addr);
                }
            }

            Instruction::NewObject {
                ty,
                ctor_name,
                args,
                register_addr,
            } => {
                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().ptr.cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();
                drop(frame);

                match cpu.new_object(ty, ctor_name, &args) {
                    Some(obj) => {
                        let frame = call_frame(cpu);
                        if !frame.write_typed(*register_addr, obj) {
                            load_register_failed!(*register_addr);
                        }
                    }
                    None => {
                        return Some(Err(Termination::NewObjectFailed));
                    }
                }
            }
            Instruction::NewArray {
                element_type,
                len,
                register_addr,
            } => {
                drop(frame);
                let Some(element_th) = element_type.load(cpu.vm_ref().assembly_manager()) else {
                    return Some(Err(Termination::LoadTypeHandleFailed(element_type.clone())));
                };
                let element_th = element_th.get_non_generic_with_method(method);
                let arr = match element_th {
                    NonGenericTypeHandle::Class(ty) => unsafe {
                        ManagedReference::alloc_array(
                            cpu,
                            ty.as_ref().method_table,
                            (*len) as usize,
                        )
                    },
                    NonGenericTypeHandle::Struct(ty) => unsafe {
                        ManagedReference::alloc_array(
                            cpu,
                            ty.as_ref().method_table,
                            (*len) as usize,
                        )
                    },
                };
                let frame = call_frame(cpu);
                if !frame.write_typed(*register_addr, arr) {
                    load_register_failed!(*register_addr);
                }
            }
            Instruction::NewDynamicArray {
                element_type,
                len_addr,
                register_addr,
            } => {
                let Some(&len) = frame.get_typed::<usize>(*len_addr) else {
                    load_register_failed!(*len_addr);
                };
                drop(frame);
                let Some(element_th) = element_type.load(cpu.vm_ref().assembly_manager()) else {
                    return Some(Err(Termination::LoadTypeHandleFailed(element_type.clone())));
                };
                let element_th = element_th.get_non_generic_with_method(method);
                let arr = match element_th {
                    NonGenericTypeHandle::Class(ty) => unsafe {
                        ManagedReference::alloc_array(cpu, ty.as_ref().method_table, len)
                    },
                    NonGenericTypeHandle::Struct(ty) => unsafe {
                        ManagedReference::alloc_array(cpu, ty.as_ref().method_table, len)
                    },
                };
                let frame = call_frame(cpu);
                if !frame.write_typed(*register_addr, arr) {
                    load_register_failed!(*register_addr);
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
                    .map(|x| frame.get(*x).unwrap().ptr.cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();

                let Some(val) = frame.read_typed::<ManagedReference<Class>>(*val) else {
                    return Some(Err(Termination::LoadRegisterFailed(*val)));
                };
                if val.is_null() {
                    return Some(Err(Termination::NullReference(
                        core::panic::Location::caller(),
                    )));
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
                    let frame = call_frame(cpu);
                    let Some(out_var) = frame.get(*ret_at) else {
                        return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                    };
                    unsafe {
                        out_var.copy_from(ret_ptr, actual_layout.size());
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
                    .map(|x| frame.get(*x).unwrap().ptr().cast::<c_void>().as_ptr())
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
                    let frame = call_frame(cpu);
                    let Some(out_var) = frame.get(*ret_at) else {
                        return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                    };
                    unsafe {
                        out_var.copy_from(ret_ptr, actual_layout.size());
                    }
                }
                unsafe {
                    std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
                }
            }
            Instruction::StaticNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => {
                let Some(&f_pointer) = frame.get_typed::<*const u8>(*f_pointer) else {
                    return Some(Err(Termination::LoadRegisterFailed(*f_pointer)));
                };
                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().ptr().cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();
                drop(frame);
                let (ret_ptr, ret_layout) = cpu.non_purus_call(config, f_pointer, args);
                let frame = call_frame(cpu);
                let Some(ret_out) = frame.get(*ret_at) else {
                    return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                };
                let true_ret_layout = config.return_type.layout();
                unsafe {
                    ret_out.copy_from(ret_ptr, true_ret_layout.size());
                    std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
                }
            }
            Instruction::DynamicNonPurusCall {
                f_pointer,
                config,
                args,
                ret_at,
            } => {
                let Some(f_pointer) = frame.read_typed::<*const u8>(*f_pointer) else {
                    return Some(Err(Termination::LoadRegisterFailed(*f_pointer)));
                };
                let Some(config) = frame.read_typed::<ManagedReference<Class>>(*config) else {
                    return Some(Err(Termination::LoadRegisterFailed(*config)));
                };
                let args = args
                    .iter()
                    .map(|x| frame.get(*x).unwrap().ptr().cast::<c_void>().as_ptr())
                    .collect::<Vec<_>>();
                drop(frame);
                let cfg = match cpu.unmarshal_non_purus_configuration(config) {
                    Ok(x) => x,
                    Err(err) => return Some(Err(Termination::UnmarshalFailed(err))),
                };
                let (ret_ptr, ret_layout) = cpu.non_purus_call(&cfg, f_pointer, args);
                let frame = call_frame(cpu);
                let Some(ret_out) = frame.get(*ret_at) else {
                    return Some(Err(Termination::LoadRegisterFailed(*ret_at)));
                };
                let true_ret_layout = cfg.return_type.layout();
                unsafe {
                    ret_out.copy_from(ret_ptr, true_ret_layout.size());
                    std::alloc::Allocator::deallocate(&std::alloc::Global, ret_ptr, ret_layout);
                }
            }

            Instruction::LoadNonPurusCallConfiguration { register_addr, val } => {
                drop(frame);

                let data = cpu.marshal_non_purus_configuration(val);

                let frame = call_frame(cpu);
                if !frame.write_typed(*register_addr, data) {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
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
                        let Some(out_var) = frame.get(*register_addr) else {
                            return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                        };
                        unsafe { out_var.copy_all_from(p.cast()) }
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
                let frame = call_frame(cpu);
                let Some(out_var) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                unsafe {
                    out_var.copy_from(f_ptr.cast(), f_layout.size());
                }
            }
            Instruction::LoadField {
                container,
                field,
                register_addr,
            } => {
                let Some(container) = frame.get(*container) else {
                    return Some(Err(Termination::LoadRegisterFailed(*container)));
                };
                let Some(register_var) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                match container.ty {
                    NonGenericTypeHandle::Class(_) => {
                        let Some((field_ptr, field_layout)) = container
                            .as_ref_typed::<ManagedReference<Class>>()
                            .const_access::<FieldAccessor<Class>>()
                            .field(*field, Default::default())
                        else {
                            return Some(Err(Termination::LoadFieldFailed(*field)));
                        };
                        debug_assert!(register_var.layout.size() >= field_layout.size());
                        unsafe {
                            register_var.copy_from(field_ptr, field_layout.size());
                        }
                    }
                    NonGenericTypeHandle::Struct(s) => {
                        let Some(field_info) = unsafe { s.as_ref() }
                            .method_table_ref()
                            .field_mem_info(*field, Default::default(), Default::default())
                        else {
                            return Some(Err(Termination::LoadFieldFailed(*field)));
                        };
                        debug_assert!(register_var.layout.size() >= field_info.layout.size());
                        unsafe {
                            register_var.copy_from(
                                container.ptr.byte_add(field_info.offset),
                                field_info.layout.size(),
                            );
                        }
                    }
                }
            }
            Instruction::SetThisField { val_addr, field } => {
                let Some(val_var) = frame.get(*val_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*val_addr)));
                };
                let Some(this) = this else {
                    return Some(Err(Termination::NullReference(
                        core::panic::Location::caller(),
                    )));
                };
                unsafe {
                    let this = this.cast::<ManagedReference<Class>>().as_ref();
                    let Some((f_ptr, f_layout)) = this
                        .const_access::<FieldAccessor<_>>()
                        .field(*field, Default::default())
                    else {
                        return Some(Err(Termination::LoadFieldFailed(*field)));
                    };
                    val_var.copy_to(f_ptr.cast(), f_layout.size());
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
                let frame = call_frame(cpu);
                let Some(val_var) = frame.get(*val_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*val_addr)));
                };
                unsafe {
                    val_var.copy_to(f_ptr.cast(), f_layout.size());
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
                let Some(res_var) = frame.get(*register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(*register_addr)));
                };
                unsafe {
                    res_var.copy_all_to(result_ptr.as_non_null_ptr());
                }
                return Some(Err(Termination::Returned));
            }

            &Instruction::Jump { target } => {
                do_jump(pc, target);
            }

            &Instruction::JumpIf {
                register_addr,
                target,
            } => {
                let Some(cond) = frame.get_typed::<bool>(register_addr) else {
                    return Some(Err(Termination::LoadRegisterFailed(register_addr)));
                };
                if *cond {
                    do_jump(pc, target);
                }
            }

            &Instruction::JumpIfAllZero { to_check, target } => {
                let Some(to_check_var) = frame.get(to_check) else {
                    return Some(Err(Termination::LoadRegisterFailed(to_check)));
                };
                if to_check_var.is_all_zero() {
                    do_jump(pc, target);
                }
            }
            &Instruction::JumpIfNotAllZero { to_check, target } => {
                let Some(to_check_var) = frame.get(to_check) else {
                    return Some(Err(Termination::LoadRegisterFailed(to_check)));
                };
                if !to_check_var.is_all_zero() {
                    do_jump(pc, target);
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
        if cpu.has_exception().unwrap() {
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
                    result_ptr
                        .as_non_null_ptr()
                        .write_bytes(0, result_ptr.len());
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

                Termination::Terminated => {}
                Termination::Returned => {}
            }
            break (result_ptr.cast(), result_layout);
        }
        pc += 1;
    }
}
