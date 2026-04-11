use std::ptr::NonNull;

use crate::{
    type_system::{class::Class, method::Method},
    virtual_machine::cpu::CPU,
};

macro get_layout($cpu:expr, $size:expr, $align:expr, $fatal:expr) {{
    let size = $size;
    let align = $align;
    let Ok(layout) = std::alloc::Layout::from_size_align(size, align) else {
        global::dt_println!("Invalid layout, size: {size}, align: {align}");
        assert!($cpu.throw_helper_mut().alloc());
        return $fatal;
    };
    layout
}}

pub extern "system" fn Global_Allocate(
    cpu: &mut CPU,
    _: &Method<Class>,

    size: usize,
    align: usize,
) -> *mut u8 {
    let layout = get_layout!(cpu, size, align, std::ptr::null_mut());

    let Ok(result) = std::alloc::Allocator::allocate(&std::alloc::Global, layout) else {
        assert!(cpu.throw_helper_mut().alloc());
        return std::ptr::null_mut();
    };

    result.as_mut_ptr()
}

pub extern "system" fn Global_AllocateZeroed(
    cpu: &mut CPU,
    _: &Method<Class>,

    size: usize,
    align: usize,
) -> *mut u8 {
    let layout = get_layout!(cpu, size, align, std::ptr::null_mut());

    let Ok(result) = std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, layout) else {
        assert!(cpu.throw_helper_mut().alloc());
        return std::ptr::null_mut();
    };

    result.as_mut_ptr()
}

pub extern "system" fn Global_Deallocate(
    cpu: &mut CPU,
    _: &Method<Class>,

    pointer: *mut u8,

    size: usize,
    align: usize,
) {
    let Some(pointer) = NonNull::new(pointer) else {
        return;
    };
    let layout = get_layout!(cpu, size, align, ());

    unsafe { std::alloc::Allocator::deallocate(&std::alloc::Global, pointer, layout) }
}

pub extern "system" fn Global_Grow(
    cpu: &mut CPU,
    _: &Method<Class>,

    pointer: *mut u8,

    old_size: usize,
    old_align: usize,

    size: usize,
    align: usize,
) -> *mut u8 {
    let Some(pointer) = NonNull::new(pointer) else {
        return std::ptr::null_mut();
    };
    let old_layout = get_layout!(cpu, old_size, old_align, std::ptr::null_mut());
    let layout = get_layout!(cpu, size, align, std::ptr::null_mut());

    let Ok(result) =
        (unsafe { std::alloc::Allocator::grow(&std::alloc::Global, pointer, old_layout, layout) })
    else {
        assert!(cpu.throw_helper_mut().alloc());
        return std::ptr::null_mut();
    };

    result.as_mut_ptr()
}

pub extern "system" fn Global_GrowZeroed(
    cpu: &mut CPU,
    _: &Method<Class>,

    pointer: *mut u8,

    old_size: usize,
    old_align: usize,

    size: usize,
    align: usize,
) -> *mut u8 {
    let Some(pointer) = NonNull::new(pointer) else {
        return std::ptr::null_mut();
    };
    let old_layout = get_layout!(cpu, old_size, old_align, std::ptr::null_mut());
    let layout = get_layout!(cpu, size, align, std::ptr::null_mut());

    let Ok(result) = (unsafe {
        std::alloc::Allocator::grow_zeroed(&std::alloc::Global, pointer, old_layout, layout)
    }) else {
        assert!(cpu.throw_helper_mut().alloc());
        return std::ptr::null_mut();
    };

    result.as_mut_ptr()
}

pub extern "system" fn Global_Shrink(
    cpu: &mut CPU,
    _: &Method<Class>,

    pointer: *mut u8,

    old_size: usize,
    old_align: usize,

    size: usize,
    align: usize,
) -> *mut u8 {
    let Some(pointer) = NonNull::new(pointer) else {
        return std::ptr::null_mut();
    };
    let old_layout = get_layout!(cpu, old_size, old_align, std::ptr::null_mut());
    let layout = get_layout!(cpu, size, align, std::ptr::null_mut());

    let Ok(result) = (unsafe {
        std::alloc::Allocator::shrink(&std::alloc::Global, pointer, old_layout, layout)
    }) else {
        assert!(cpu.throw_helper_mut().alloc());
        return std::ptr::null_mut();
    };

    result.as_mut_ptr()
}
