use std::{
    alloc::Layout,
    ffi::{CString, c_void},
    ptr::NonNull,
};

use allocate_guard::{AllocateGuard, GuardedBox};
use global::{
    attrs::CallConvention,
    non_purus_call_configuration::{
        NonPurusCallConfiguration, NonPurusCallType, ObjectStrategy, StringEncoding,
    },
};
use stdlib_header::CoreTypeId;

use crate::{
    error::RuntimeError,
    type_system::class::Class,
    value::managed_reference::{ArrayAccessor, FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

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
    (is_by_ref, ty): (bool, &NonPurusCallType),
) -> libffi::middle::Type {
    use libffi::middle::Type as LibffiType;
    if is_by_ref {
        LibffiType::pointer()
    } else {
        non_purus_type_to_libffi_type(ty)
    }
}

// Marshal NonPurus*
impl CPU {
    pub fn marshal_non_purus_type(&mut self, ty: &NonPurusCallType) -> ManagedReference<Class> {
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
                stdlib_header::FieldId!(NonPurusCallType::Discriminant) as _,
                Default::default(),
            )
            .unwrap() = ty.discriminant();
        match ty {
            NonPurusCallType::Structure(fields) => {
                let arr = {
                    let slice = fields
                        .iter()
                        .map(|x| self.marshal_non_purus_type(x))
                        .collect();
                    ManagedReference::new_array::<_, ManagedReference<Class>>(self, mt, slice)
                };
                *obj.const_access_mut::<FieldAccessor<Class>>()
                    .typed_field_mut(
                        stdlib_header::FieldId!(NonPurusCallType::Types) as _,
                        Default::default(),
                    )
                    .unwrap() = arr;
            }
            _ => (),
        }
        obj
    }
    pub fn marshal_non_purus_configuration(
        &mut self,
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

        macro write_field($T:ty: $val:expr => $field:ident) {
            assert!(obj_fields.write_typed_field::<$T>(
                ::stdlib_header::System::NonPurusCallConfiguration::FieldId::$field as _,
                Default::default(),
                $val,
            ))
        }
        macro write_u8($val:expr => $field:ident) {
            write_field!(u8: $val => $field)
        }

        write_u8!(cfg.call_convention.into() => CallConvention);

        write_field!(ManagedReference<Class>: self.marshal_non_purus_type(&cfg.return_type) => ReturnType);

        write_u8!(cfg.encoding.into() => Encoding);

        write_u8!(cfg.object_strategy.into() => ObjectStrategy);

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
        write_field!(ManagedReference<Class>: m_by_ref_args => ByRefArguments);
        write_field!(ManagedReference<Class>: m_args => Arguments);

        obj
    }
}

// Unmarshal NonPurus*
impl CPU {
    pub fn unmarshal_non_purus_type(ty: ManagedReference<Class>) -> Option<NonPurusCallType> {
        let accessor = ty.const_access::<FieldAccessor<Class>>();
        let discriminant = accessor.read_typed_field::<u8>(
            stdlib_header::FieldId!(NonPurusCallType::Discriminant) as _,
            Default::default(),
        )?;
        NonPurusCallType::from_u8(discriminant, || {
            let field_types = accessor.typed_field::<ManagedReference<Class>>(
                stdlib_header::FieldId!(NonPurusCallType::Types) as _,
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

        macro get_field($field:ident: $T:ty) {
            accessor
                .read_typed_field::<$T>(
                    ::stdlib_header::System::NonPurusCallConfiguration::FieldId::$field as _,
                    Default::default(),
                )
                .ok_or(RuntimeError::FailedGetField(
                    ::stdlib_header::System::NonPurusCallConfiguration::FieldId::$field as _,
                ))?
        }

        macro access_u8($val:ident: $ty:ty) {
            <$ty>::try_from(
                accessor
                    .read_typed_field::<u8>(
                        ::stdlib_header::System::NonPurusCallConfiguration::FieldId::$val as _,
                        Default::default(),
                    )
                    .ok_or(RuntimeError::FailedGetField(
                        ::stdlib_header::System::NonPurusCallConfiguration::FieldId::$val as _,
                    ))?,
            )?
        }

        let call_convention = access_u8!(CallConvention: CallConvention);

        let return_type =
            Self::unmarshal_non_purus_type(get_field!(ReturnType: ManagedReference<Class>))
                .ok_or(RuntimeError::unmarshal_failed::<NonPurusCallType>())?;

        let encoding = access_u8!(Encoding: StringEncoding);

        let object_strategy = access_u8!(ObjectStrategy: ObjectStrategy);

        let by_ref_args = get_field!(ByRefArguments: ManagedReference<Class>);

        let in_sign_args = get_field!(Arguments: ManagedReference<Class>);

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
}

#[derive(Clone, PartialEq, Eq)]
pub enum NonPurusCallArgType {
    String,
    Object,
    Other,
}

#[derive(Clone)]
pub struct NonPurusCallArg {
    pub val: *mut c_void,
    pub ty: NonPurusCallType,
}

impl NonPurusCallArg {
    pub fn new<T>(val: *const T, ty: NonPurusCallType) -> Self {
        Self {
            val: val.cast_mut().cast(),
            ty,
        }
    }
}

impl CPU {
    pub fn dynamic_non_purus_call(
        &self,
        cfg: ManagedReference<Class>,
        f_ptr: *const u8,
        args: Vec<NonPurusCallArg>,
    ) -> global::Result<(NonNull<u8>, Layout)> {
        self.unmarshal_non_purus_configuration(cfg)
            .map(|cfg| self.non_purus_call(&cfg, f_ptr, args))
    }
    pub fn non_purus_call(
        &self,
        cfg: &NonPurusCallConfiguration,
        f_ptr: *const u8,
        mut args: Vec<NonPurusCallArg>,
    ) -> (NonNull<u8>, Layout) {
        use libffi::middle::Cif;

        let abi = crate::libffi_utils::get_abi_by_call_convention(cfg.call_convention);
        let cif = if cfg.arguments.len() < args.len() {
            #[cfg(unix)]
            println!("Variadic functions are not yet fully supported");
            let args = args
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    if let Some((is_by_ref, _)) = cfg.arguments.get(i)
                        && *is_by_ref
                    {
                        (true, &x.ty)
                    } else {
                        (false, &x.ty)
                    }
                })
                .map(non_purus_type_arg_to_libffi_type);
            let fixed_args = cfg.arguments.len();
            Cif::new_variadic_with_abi(
                args,
                fixed_args,
                non_purus_type_to_libffi_type(&cfg.return_type),
                abi,
            )
        } else if cfg.arguments.len() == args.len() {
            Cif::new_with_abi(
                cfg.arguments
                    .iter()
                    .map(|x| (x.0, &x.1))
                    .map(non_purus_type_arg_to_libffi_type),
                non_purus_type_to_libffi_type(&cfg.return_type),
                abi,
            )
        } else {
            panic!();
        };
        let allocate_guard = AllocateGuard::global();
        let mut treat_string_as_object = false;
        match cfg.encoding {
            global::non_purus_call_configuration::StringEncoding::Utf8 => {
                for (i, arg) in args.iter_mut().enumerate() {
                    if cfg.arguments.get(i).is_some_and(|x| x.0) {
                        continue;
                    }

                    if arg.ty == NonPurusCallType::String {
                        let data =
                            unsafe { arg.val.cast::<ManagedReference<Class>>().as_ref_unchecked() }
                                .access::<StringAccessor>()
                                .unwrap()
                                .to_string()
                                .unwrap()
                                .unwrap()
                                .into_boxed_str();
                        let data = GuardedBox::into_non_null(GuardedBox::clone_from_ref(
                            &*data,
                            &allocate_guard,
                        ));
                        let data_ptr = GuardedBox::into_non_null(GuardedBox::new(
                            data.as_ptr().cast_const().cast::<u8>(),
                            &allocate_guard,
                        ));
                        arg.val = data_ptr.cast().as_ptr();
                    }
                }
            }
            global::non_purus_call_configuration::StringEncoding::C_Utf16
            | global::non_purus_call_configuration::StringEncoding::Utf16 => {
                for (i, arg) in args.iter_mut().enumerate() {
                    if cfg.arguments.get(i).is_some_and(|x| x.0) {
                        continue;
                    }

                    if arg.ty == NonPurusCallType::String {
                        let ptr = GuardedBox::into_non_null(GuardedBox::new(
                            unsafe {
                                arg.val
                                    .cast::<ManagedReference<Class>>()
                                    .as_ref_unchecked()
                                    .data_ptr()
                                    .cast_const()
                                    .cast::<u16>()
                            },
                            &allocate_guard,
                        ));
                        arg.val = ptr.cast().as_ptr();
                    }
                }
            }
            global::non_purus_call_configuration::StringEncoding::C_Utf8 => {
                for (i, arg) in args.iter_mut().enumerate() {
                    if cfg.arguments.get(i).is_some_and(|x| x.0) {
                        continue;
                    }

                    if arg.ty == NonPurusCallType::String {
                        let data = CString::new(
                            unsafe { arg.val.cast::<ManagedReference<Class>>().as_ref_unchecked() }
                                .access::<StringAccessor>()
                                .unwrap()
                                .to_string()
                                .unwrap()
                                .unwrap(),
                        )
                        .unwrap()
                        .into_boxed_c_str();
                        let data = GuardedBox::into_non_null(GuardedBox::clone_from_ref(
                            &*data,
                            &allocate_guard,
                        ));
                        #[cfg(windows)]
                        {
                            let ptr = GuardedBox::into_non_null(GuardedBox::new(
                                data.as_ptr().cast_const().cast::<std::ffi::c_char>(),
                                &allocate_guard,
                            ));
                            arg.val = ptr.cast().as_ptr();
                        }
                        #[cfg(not(windows))]
                        {
                            arg.val = data.cast().as_ptr();
                        }
                    }
                }
            }
            global::non_purus_call_configuration::StringEncoding::Remain => {
                treat_string_as_object = true;
            }
        }
        match cfg.object_strategy {
            ObjectStrategy::Remain => (),
            ObjectStrategy::PointToData => {
                for (i, arg) in args.iter_mut().enumerate() {
                    if cfg.arguments.get(i).is_some_and(|x| x.0) {
                        continue;
                    }

                    if (arg.ty == NonPurusCallType::Object)
                        || ((arg.ty == NonPurusCallType::String) && treat_string_as_object)
                    {
                        let ptr = GuardedBox::into_non_null(GuardedBox::new(
                            unsafe { arg.val.cast::<ManagedReference<Class>>().read().data_ptr() },
                            &allocate_guard,
                        ));
                        arg.val = ptr.as_ptr().cast();
                    }
                }
            }
        }
        for (i, arg) in args.iter_mut().enumerate() {
            if cfg.arguments.get(i).is_some_and(|x| x.0) {
                let p = GuardedBox::into_non_null(GuardedBox::new(arg.val, &allocate_guard));
                arg.val = p.as_ptr().cast();
            }
        }
        let fun = libffi::low::CodePtr::from_ptr(f_ptr.cast());
        let result_layout = crate::memory::get_return_layout_for_libffi(cfg.return_type.layout());
        let result_ptr =
            std::alloc::Allocator::allocate_zeroed(&std::alloc::Global, result_layout).unwrap();

        let mut args_to_pass = args.iter().map(|x| x.val).collect::<Vec<_>>();

        unsafe {
            libffi::raw::ffi_call(
                cif.as_raw_ptr(),
                Some(*fun.as_safe_fun()),
                result_ptr.as_non_null_ptr().cast::<c_void>().as_ptr(),
                args_to_pass.as_mut_ptr(),
            );
        }

        (result_ptr.as_non_null_ptr(), result_layout)
    }
}
