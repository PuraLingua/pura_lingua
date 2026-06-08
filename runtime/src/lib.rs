/* cSpell:disable */
#![feature(box_vec_non_null)]
#![feature(mapped_lock_guards)]
#![feature(allocator_api)]
#![feature(layout_for_ptr)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_try)]
#![feature(current_thread_id)]
#![feature(likely_unlikely)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]
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
#![feature(more_qualified_paths)]
#![feature(try_find)]
#![feature(impl_trait_in_bindings)]
#![feature(exitcode_exit_method)]
#![feature(extern_types)]
#![feature(macro_metavar_expr_concat)]
#![feature(macro_metavar_expr)]
#![feature(doc_cfg)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(sync_unsafe_cell)]
#![feature(box_into_inner)]
#![feature(pin_ergonomics)]
#![feature(extend_one)]
#![feature(super_let)]
#![feature(const_range)]
#![feature(trivial_clone)]
#![feature(sized_hierarchy)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(panic_internals)]
#![feature(variant_count)]
#![feature(const_ops)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]
#![feature(nonpoison_rwlock)]
#![feature(dispatch_from_dyn)]
#![cfg_attr(test, feature(nonpoison_condvar))]
#![cfg_attr(test, feature(vec_from_fn))]
#![cfg_attr(all(unix, test), feature(cstr_display))]
#![cfg_attr(all(unix, test), feature(c_variadic))]
/* cSpell:enable */
#![allow(internal_features, incomplete_features)]
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
pub(crate) mod utils;

#[cfg(test)]
mod global_tests;
