use std::{ffi::c_void, mem::offset_of, pin::Pin, ptr::NonNull};

use global::{
    attrs::{CallConvention, MethodAttr, MethodImplementationFlags},
    getset::Getters,
    instruction::Instruction,
};
use libffi::low::CodePtr;

use crate::{
    stdlib::{CoreTypeId, CoreTypeIdConstExt as _},
    type_system::{
        generics::GenericBounds, method_table::MethodTable, type_handle::MaybeUnloadedTypeHandle,
    },
    virtual_machine::cpu::CPU,
};

use super::{
    get_traits::{GetAssemblyRef, GetTypeVars},
    type_handle::NonGenericTypeHandle,
};

mod calling;
mod exception_table;
mod parameter;

pub use exception_table::{ExceptionTable, ExceptionTableEntry};
pub use parameter::Parameter;

pub type RuntimeInstruction = Instruction<String, MaybeUnloadedTypeHandle, MethodRef, u32>;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Method<T> {
    #[getset(skip)]
    pub(crate) mt: Option<NonNull<MethodTable<T>>>,
    generic: Option<NonNull<Self>>,

    name: Box<str>,
    attr: MethodAttr<MaybeUnloadedTypeHandle>,
    args: Vec<Parameter>,
    return_type: MaybeUnloadedTypeHandle,
    #[getset(skip)]
    call_convention: CallConvention,

    generic_instances: Vec<NonNull<Self>>,
    generic_bounds: Option<NonNull<[GenericBounds]>>,
    type_vars: Option<Box<[MaybeUnloadedTypeHandle]>>,

    instructions: Vec<RuntimeInstruction>,
    entry_point: CodePtr,

    exception_table: ExceptionTable<T>,
}

mod display;

pub use display::MethodDisplayOptions;

pub(crate) mod default_entry_point;

impl<T> Method<T> {
    pub const fn require_method_table_ref(&self) -> &MethodTable<T> {
        unsafe { self.mt.unwrap().as_ref() }
    }
}

#[allow(clippy::too_many_arguments)]
impl<T> Method<T>
where
    T: GetTypeVars + GetAssemblyRef,
{
    pub fn new<FExceptionTable: FnOnce(&Self) -> ExceptionTable<T>>(
        mt: NonNull<MethodTable<T>>,

        name: String,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
        args: Vec<Parameter>,
        return_type: MaybeUnloadedTypeHandle,
        call_convention: CallConvention,

        generic_bounds: Option<Vec<GenericBounds>>,

        instructions: Vec<RuntimeInstruction>,

        exception_table_generator: FExceptionTable,
    ) -> Pin<Box<Self>> {
        let mut this = Box::pin(Self {
            mt: Some(mt),
            generic: None,

            name: name.into_boxed_str(),
            attr,
            args,
            return_type,
            call_convention,

            generic_instances: Vec::new(),
            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,

            instructions,
            entry_point: CodePtr::from_ptr(default_entry_point::__default_entry_point::<T> as _),

            exception_table: ExceptionTable::new(NonNull::dangling()),
        });
        this.exception_table = exception_table_generator(&this);
        this
    }

    pub fn try_new<E, FExceptionTable: FnOnce(&Self) -> Result<ExceptionTable<T>, E>>(
        mt: NonNull<MethodTable<T>>,

        name: String,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
        args: Vec<Parameter>,
        return_type: MaybeUnloadedTypeHandle,
        call_convention: CallConvention,

        generic_bounds: Option<Vec<GenericBounds>>,

        instructions: Vec<RuntimeInstruction>,

        exception_table_generator: FExceptionTable,
    ) -> Result<Pin<Box<Self>>, E> {
        let mut this = Box::pin(Self {
            mt: Some(mt),
            generic: None,

            name: name.into_boxed_str(),
            attr,
            args,
            return_type,
            call_convention,

            generic_instances: Vec::new(),
            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,

            instructions,
            entry_point: CodePtr::from_ptr(default_entry_point::__default_entry_point::<T> as _),

            exception_table: ExceptionTable::new(NonNull::dangling()),
        });
        this.exception_table = exception_table_generator(&this)?;
        Ok(this)
    }
}

#[allow(clippy::too_many_arguments)]
impl<T> Method<T> {
    pub fn native<FExceptionTable: FnOnce(&Self) -> ExceptionTable<T>>(
        mt: Option<NonNull<MethodTable<T>>>,

        name: String,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
        args: Vec<Parameter>,
        return_type: MaybeUnloadedTypeHandle,
        call_convention: CallConvention,

        generic_bounds: Option<Vec<GenericBounds>>,

        entry_point: *const c_void,

        exception_table_generator: FExceptionTable,
    ) -> Pin<Box<Self>> {
        let mut this = Box::pin(Self {
            mt,
            generic: None,

            name: name.into_boxed_str(),
            attr,
            args,
            return_type,
            call_convention,

            generic_instances: Vec::new(),
            generic_bounds: generic_bounds
                .filter(|x| !x.is_empty())
                .map(|x| Box::into_non_null(x.into_boxed_slice())),
            type_vars: None,

            instructions: Vec::new(),
            entry_point: CodePtr::from_ptr(entry_point),

            exception_table: ExceptionTable::new(NonNull::dangling()),
        });
        this.exception_table = exception_table_generator(&this);
        this
    }
    /// Creates a static method called `.sctor`
    pub fn default_sctor(
        mt: Option<NonNull<MethodTable<T>>>,
        attr: MethodAttr<MaybeUnloadedTypeHandle>,
    ) -> Pin<Box<Self>> {
        extern "system" fn sctor<T>(_: &mut CPU, _: &Method<T>) {
            #[cfg(feature = "print_invoke_and_call")]
            global::dt_println!("Calling default sctor");
        }
        Self::create_sctor(mt, attr, sctor::<T>)
    }
    /// Creates a static method called `.sctor`
    pub fn create_sctor(
        mt: Option<NonNull<MethodTable<T>>>,
        mut attr: MethodAttr<MaybeUnloadedTypeHandle>,
        rust_fn: extern "system" fn(&mut CPU, &Method<T>),
    ) -> Pin<Box<Self>> {
        attr.impl_flags_mut()
            .insert(MethodImplementationFlags::Static);
        Self::native(
            mt,
            ".sctor".to_owned(),
            attr,
            vec![],
            CoreTypeId::System_Void.static_type_ref().into(),
            CallConvention::PlatformDefault,
            None,
            rust_fn as _,
            ExceptionTable::gen_new(),
        )
    }
}

impl<T> Method<T> {
    pub fn instantiate(&self, type_vars: &[MaybeUnloadedTypeHandle]) -> NonNull<Self> {
        for has_instantiated in self.generic_instances.iter() {
            if unsafe { has_instantiated.as_ref() }
                .type_vars
                .as_deref()
                .is_some_and(|x| x.eq(type_vars))
            {
                return *has_instantiated;
            }
        }

        let instantiated = Box::new(Self {
            mt: self.mt,
            generic: Some(NonNull::from_ref(self)),

            name: self.name.clone(),
            attr: self.attr.clone(),

            args: self.args.clone(),
            return_type: self.return_type.clone(),
            call_convention: self.call_convention,
            instructions: self.instructions.clone(),
            entry_point: self.entry_point,

            generic_instances: Vec::new(),
            generic_bounds: None,
            type_vars: Some(Box::clone_from_ref(type_vars)),

            exception_table: self.exception_table.clone(),
        });

        let mut instantiated = Box::into_non_null(instantiated);

        unsafe {
            instantiated
                .as_mut()
                .exception_table
                .reset_method_ptr(instantiated);
            NonNull::from_ref(self)
                .byte_add(offset_of!(Self, generic_instances))
                .cast::<Vec<NonNull<Self>>>()
                .as_mut()
                .push(instantiated);
        }

        instantiated
    }
}

impl<T> Method<T> {
    pub const fn call_convention(&self) -> CallConvention {
        self.call_convention
    }
    const fn libffi_call_convention(&self) -> libffi::middle::FfiAbi {
        crate::libffi_utils::get_abi_by_call_convention(self.call_convention)
    }
}

impl<T: GetTypeVars + GetAssemblyRef> Method<T> {
    pub fn get_return_type(&self) -> NonGenericTypeHandle {
        match &self.return_type {
            MaybeUnloadedTypeHandle::Loaded(ty) => ty.get_non_generic_with_method(self),
            MaybeUnloadedTypeHandle::Unloaded(_) => {
                let ty = self
                    .return_type
                    .load(unsafe {
                        self.mt
                            .unwrap()
                            .as_ref()
                            .ty_ref()
                            .__get_assembly_ref()
                            .manager_ref()
                    })
                    .unwrap();
                // Hacking
                unsafe {
                    NonNull::from_ref(self).as_mut().return_type =
                        MaybeUnloadedTypeHandle::Loaded(ty);
                }

                ty.get_non_generic_with_method(self)
            }
        }
    }
    fn libffi_return_type(&self) -> libffi::middle::Type {
        if self
            .get_return_type()
            .is_certain_core_type(CoreTypeId::System_Void)
        {
            libffi::middle::Type::void()
        } else {
            self.get_return_type().val_libffi_type()
        }
    }
}

const _: () = {
    use global::assertions::*;
    fn _assert()
    where
        for<'a> LayoutEq<libffi::middle::Arg<'a>, *mut c_void>: SuccessAssert,
    {
    }
};

#[derive(Clone, Debug)]
pub enum MethodRef {
    Index(u32),
    Specific {
        index: u32,
        types: Vec<MaybeUnloadedTypeHandle>,
    },
}

impl MethodRef {
    pub fn index(&self) -> u32 {
        match self {
            MethodRef::Index(index) => *index,
            MethodRef::Specific { index, .. } => *index,
        }
    }
    pub fn map_index(self, f: impl FnOnce(u32) -> u32) -> Self {
        match self {
            MethodRef::Index(index) => Self::Index(f(index)),
            MethodRef::Specific { index, types } => Self::Specific {
                index: f(index),
                types,
            },
        }
    }
    pub fn cloned_map_index(&self, f: impl FnOnce(u32) -> u32) -> Self {
        match self {
            MethodRef::Index(index) => Self::Index(f(*index)),
            MethodRef::Specific { index, types } => Self::Specific {
                index: f(*index),
                types: types.clone(),
            },
        }
    }
}

impl std::fmt::Display for MethodRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodRef::Index(i) => write!(f, "INDEX({i})"),
            MethodRef::Specific { index, types } => write!(f, "{index}<{types:?}>"),
        }
    }
}

#[cfg(test)]
mod tests;
