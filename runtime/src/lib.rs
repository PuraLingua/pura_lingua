#![feature(box_vec_non_null)]
#![feature(mapped_lock_guards)]
#![feature(str_as_str)]
#![feature(allocator_api)]
#![feature(layout_for_ptr)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_try)]
#![feature(current_thread_id)]
#![feature(likely_unlikely)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]
#![feature(ptr_internals)]
#![feature(const_convert)]
#![feature(const_clone)]
#![feature(clone_from_ref)]
#![feature(core_intrinsics)]
#![feature(ptr_alignment_type)]
#![feature(slice_ptr_get)]
#![feature(specialization)]
#![feature(lock_value_accessors)]
#![feature(const_option_ops)]
#![feature(const_cmp)]
#![feature(const_cell_traits)]
#![feature(const_eval_select)]
#![feature(iterator_try_collect)]
#![feature(ptr_metadata)]
#![feature(const_result_trait_fn)]
#![feature(derive_const)]
#![feature(generic_const_exprs)]
#![feature(pointer_is_aligned_to)]
#![feature(impl_trait_in_bindings)]
/* cSpell:disable-next-line */
#![feature(exitcode_exit_method)]
#![feature(extern_types)]
/* cSpell:disable-next-line */
#![feature(macro_metavar_expr_concat)]
/* cSpell:disable-next-line */
#![feature(macro_metavar_expr)]
#![feature(doc_cfg)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(sync_unsafe_cell)]
#![feature(box_into_inner)]
#![feature(pin_ergonomics)]
#![cfg_attr(all(unix, test), feature(cstr_display))]
//
#![allow(static_mut_refs)]
#![allow(internal_features, incomplete_features)]
#![allow(clippy::mut_from_ref)]
#![allow(clippy::get_first)]
#![allow(clippy::new_without_default)]
#![allow(clippy::vec_box)]

pub mod error;
pub mod memory;
pub mod stdlib;
#[cfg(test)]
pub mod test_utils;
pub mod type_system;
pub mod value;
pub mod virtual_machine;

pub(crate) mod c_ffi;
pub(crate) mod libffi_utils;
pub(crate) mod llvm_utils;

#[cfg(test)]
mod global_tests;
