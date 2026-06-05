use std::{pin::Pin, ptr::NonNull, sync::atomic::AtomicU8};

use either::Either;
use global::StringName;

use crate::type_system::{
    assembly::{Assembly, TypeContainer},
    assembly_manager::AssemblyRef,
    cached_type_reference::CachedTypeReference,
    class::{Class, ClassParent, LoadedClassParent},
    field::Field,
    generics::{GenericBounds, GenericCountRequirement},
    get_traits::{GetAssemblyRef, GetTypeVars},
    interface::{Interface, InterfaceImplementation},
    method::{ExceptionTable, ExceptionTableEntry, Method, MethodRef, Parameter},
    method_table::MethodTable,
    r#struct::Struct,
    type_handle::{GenericUnresolvable, MaybeUnloadedTypeHandle, NonGenericTypeHandle, TypeHandle},
    type_ref::TypeRef,
};

use super::AssemblyManager;

#[derive(Debug)]
pub struct AtomicTypeLoadState(AtomicU8);

mod atomic_type_load_state {
    use std::{mem::transmute, sync::atomic::Ordering};

    use super::TypeLoadState;

    #[inline(always)]
    const fn safe_transmute(x: u8) -> TypeLoadState {
        unsafe { transmute(x) }
    }

    impl super::AtomicTypeLoadState {
        #[inline(always)]
        pub fn load(&self, order: Ordering) -> TypeLoadState {
            unsafe { transmute(self.0.load(order)) }
        }
        #[inline(always)]
        pub fn store(&self, val: TypeLoadState, order: Ordering) {
            self.0.store(val as u8, order);
        }
        #[inline(always)]
        pub fn swap(&self, val: TypeLoadState, order: Ordering) -> TypeLoadState {
            unsafe { transmute(self.0.swap(val as u8, order)) }
        }
        #[inline(always)]
        pub fn compare_exchange(
            &self,
            current: TypeLoadState,
            new: TypeLoadState,
            success: Ordering,
            failure: Ordering,
        ) -> Result<TypeLoadState, TypeLoadState> {
            self.0
                .compare_exchange(current as u8, new as u8, success, failure)
                .map(safe_transmute)
                .map_err(safe_transmute)
        }

        #[inline(always)]
        pub fn compare_exchange_weak(
            &self,
            current: TypeLoadState,
            new: TypeLoadState,
            success: Ordering,
            failure: Ordering,
        ) -> Result<TypeLoadState, TypeLoadState> {
            self.0
                .compare_exchange_weak(current as u8, new as u8, success, failure)
                .map(safe_transmute)
                .map_err(safe_transmute)
        }

        #[inline(always)]
        pub fn try_update<F>(
            &self,
            set_order: Ordering,
            fetch_order: Ordering,
            mut f: impl FnMut(TypeLoadState) -> Option<TypeLoadState>,
        ) -> Result<TypeLoadState, TypeLoadState> {
            #[inline(always)]
            const fn map(x: TypeLoadState) -> u8 {
                x as u8
            }
            self.0
                .try_update(set_order, fetch_order, move |x| {
                    f(unsafe { transmute(x) }).map(map)
                })
                .map(safe_transmute)
                .map_err(safe_transmute)
        }

        #[inline(always)]
        pub fn update(
            &self,
            set_order: Ordering,
            fetch_order: Ordering,
            mut f: impl FnMut(TypeLoadState) -> TypeLoadState,
        ) -> TypeLoadState {
            unsafe {
                transmute(
                    self.0
                        .update(set_order, fetch_order, move |x| f(transmute(x)) as u8),
                )
            }
        }

        #[inline(always)]
        pub const fn as_ptr(&self) -> *mut TypeLoadState {
            self.0.as_ptr().cast()
        }
    }
}

impl AtomicTypeLoadState {
    #[inline(always)]
    pub const fn new(v: TypeLoadState) -> Self {
        Self(AtomicU8::new(v as u8))
    }
    pub fn is_finished(&self) -> bool {
        self.load(std::sync::atomic::Ordering::Acquire) == TypeLoadState::Finished
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeLoadState {
    Loading,
    Finished,
}

impl AssemblyManager {
    pub fn load_binaries(
        &self,
        binaries: &[binary::assembly::Assembly],
    ) -> binary::prelude::BinaryResult<()> {
        let mut loaded_ids = Vec::new();
        for b_assembly in binaries {
            loaded_ids.push(self.load_binary(b_assembly)?);
        }

        for (b_assembly_id, loaded_id) in loaded_ids.into_iter().enumerate() {
            let assembly = self.get_assembly(loaded_id).unwrap().unwrap();
            let b_assembly = &binaries[b_assembly_id];
            let types = assembly.types.read().unwrap();
            for (t_id, ty) in types.iter().enumerate() {
                match ty {
                    TypeContainer::Class(class) => {
                        if class.load_state.is_finished() {
                            continue;
                        }
                        let Some(parent) = (unsafe { NonNull::from_ref(&class.m_parent).as_mut() })
                        else {
                            continue;
                        };
                        let parent_type_ref = unsafe { Box::from_non_null(parent.unloaded) };
                        *parent = match Box::into_inner(parent_type_ref) {
                            TypeRef::Index { assembly, ind } => {
                                let assembly = self.get_assembly_by_ref(&assembly).unwrap().ok_or(
                                    binary::binary_core::Error::UnknownAssembly(
                                        assembly.to_string(),
                                    ),
                                )?;
                                ClassParent::new_simple(
                                    assembly
                                        .get_class(ind)
                                        .ok_or(binary::binary_core::Error::UnknownType(ind))?,
                                )
                            }
                            TypeRef::Specific {
                                assembly_and_index,
                                types,
                            } => {
                                let to_instantiate = match assembly_and_index {
                                    Either::Left((assembly, ind)) => {
                                        let assembly = self
                                            .get_assembly_by_ref(&assembly)
                                            .unwrap()
                                            .ok_or(binary::binary_core::Error::UnknownAssembly(
                                                assembly.to_string(),
                                            ))?;
                                        assembly
                                            .get_class(ind)
                                            .ok_or(binary::binary_core::Error::UnknownType(ind))?
                                    }
                                    Either::Right(x) => x
                                        .load_with_generic_resolver(self, &GenericUnresolvable)
                                        .unwrap()
                                        .into_non_generic()
                                        .unwrap()
                                        .unwrap_class(),
                                };
                                ClassParent::new_with_generic(to_instantiate, types)
                            }
                        };

                        (*unsafe { NonNull::from_ref(&class.method_table).as_mut() }) =
                            MethodTable::new(NonNull::from_ref(&**class), |mt| {
                                self.load_binary_methods(
                                    &assembly,
                                    loaded_id,
                                    b_assembly,
                                    t_id as _,
                                    mt,
                                    &b_assembly.type_defs[t_id].unwrap_class_ref().method_table,
                                )
                                .unwrap()
                            })
                            .as_non_null_ptr();
                        unsafe { NonNull::from_ref(class).as_mut() }
                            .rediscover_sctor(b_assembly.type_defs[t_id].unwrap_class_ref().sctor);

                        class.load_state.store(
                            TypeLoadState::Finished,
                            std::sync::atomic::Ordering::Release,
                        );
                    }
                    TypeContainer::Struct(_) => {}
                    TypeContainer::Interface(_) => {}
                }
            }
        }

        Ok(())
    }

    pub fn load_binary(
        &self,
        binary: &binary::assembly::Assembly,
    ) -> binary::binary_core::BinaryResult<usize> {
        let name = binary.get_string(binary.extra_header.name)?;
        let id = self.add_assembly(Box::new(Assembly::new(
            self,
            widestring::Utf16String::from_str(name),
            false,
        )));
        let assembly = self.get_assembly(id).unwrap().unwrap();
        for (type_id, type_def) in binary.type_defs.iter().enumerate() {
            match type_def {
                binary::ty::TypeDef::Class(class_def) => {
                    self.load_binary_class(&assembly, id, binary, class_def, type_id as u32)?;
                }
                binary::ty::TypeDef::Struct(struct_def) => {
                    self.load_binary_struct(&assembly, id, binary, struct_def, type_id as u32)?;
                }
                binary::ty::TypeDef::Interface(interface_def) => {
                    self.load_binary_interface(
                        &assembly,
                        id,
                        binary,
                        interface_def,
                        type_id as u32,
                    )?;
                }
            }
        }

        Ok(id)
    }

    fn load_binary_class(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        class_def: &binary::ty::ClassDef,
        class_id: u32,
    ) -> binary::prelude::BinaryResult<()> {
        let name = b_assembly.get_string(class_def.name)?;
        let mut parent_loaded = true;
        let result = Class::new_for_binary(
            NonNull::from_ref(assembly),
            class_def.main,
            widestring::Utf16String::from_str(name),
            class_def.attr,
            class_def.generic_count_requirement.into(),
            class_def
                .parent
                .as_ref()
                .map(|x| {
                    let parent = match MaybeUnloadedTypeHandle::from_token_for_type(
                        assembly,
                        assembly_id,
                        b_assembly,
                        x,
                        class_id,
                    ) {
                        Ok(x) => x,
                        Err(e) => return Err(e),
                    };
                    match parent {
                        MaybeUnloadedTypeHandle::Loaded(type_handle) => {
                            match type_handle.into_non_generic() {
                                None => Err(binary::prelude::Error::InheritFromGeneric),
                                Some(NonGenericTypeHandle::Class(class)) => {
                                    Ok(Either::Left(LoadedClassParent::Simple(class)))
                                }
                                Some(_) => Err(binary::prelude::Error::WrongParentType),
                            }
                        }
                        MaybeUnloadedTypeHandle::Unloaded(type_ref) => {
                            parent_loaded = false;
                            Ok(Either::Right(type_ref))
                        }
                    }
                })
                .transpose()?,
            |rt_class| {
                if !parent_loaded {
                    return NonNull::dangling();
                }
                MethodTable::new(rt_class, |mt| {
                    self.load_binary_methods(
                        assembly,
                        assembly_id,
                        b_assembly,
                        class_id,
                        mt,
                        &class_def.method_table,
                    )
                    .unwrap()
                })
                .as_non_null_ptr()
            },
            class_def
                .fields
                .iter()
                .map(|field| {
                    self.load_binary_field(assembly, assembly_id, b_assembly, class_id, field)
                })
                .try_collect()?,
            class_def.sctor,
            class_def
                .interfaces
                .iter()
                .map(|x| {
                    MaybeUnloadedTypeHandle::from_token_for_type(
                        assembly,
                        assembly_id,
                        b_assembly,
                        &x.target,
                        class_id,
                    )
                    .map(|target| InterfaceImplementation {
                        target,
                        map: x.map.clone(),
                    })
                })
                .try_collect()?,
            self.load_binary_generic_bounds(
                assembly,
                assembly_id,
                b_assembly,
                class_id,
                &class_def.generic_bounds,
            )?,
        );

        assert_eq!(assembly.add_type(result), class_id);
        Ok(())
    }

    fn load_binary_struct(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        struct_def: &binary::ty::StructDef,
        struct_id: u32,
    ) -> binary::binary_core::BinaryResult<()> {
        let name = b_assembly.get_string(struct_def.name)?;
        let result = Struct::new(
            NonNull::from_ref(assembly),
            widestring::Utf16String::from_str(name),
            struct_def.attr,
            struct_def.generic_count_requirement.into(),
            |rt_struct| {
                MethodTable::new(rt_struct, |mt| {
                    self.load_binary_methods(
                        assembly,
                        assembly_id,
                        b_assembly,
                        struct_id,
                        mt,
                        &struct_def.method_table,
                    )
                    .unwrap()
                })
                .as_non_null_ptr()
            },
            struct_def
                .fields
                .iter()
                .map(|field| {
                    self.load_binary_field(assembly, assembly_id, b_assembly, struct_id, field)
                })
                .try_collect()?,
            struct_def.sctor,
            self.load_binary_generic_bounds(
                assembly,
                assembly_id,
                b_assembly,
                struct_id,
                &struct_def.generic_bounds,
            )?,
        );

        assert_eq!(assembly.add_type(result), struct_id);
        Ok(())
    }

    fn load_binary_interface(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        interface_def: &binary::ty::InterfaceDef,
        interface_id: u32,
    ) -> binary::binary_core::BinaryResult<()> {
        let name = b_assembly.get_string(interface_def.name)?;
        let result = Interface::new(
            NonNull::from_ref(assembly),
            widestring::Utf16String::from_str(name),
            interface_def.attr,
            GenericCountRequirement::default(),
            interface_def
                .required_interfaces
                .iter()
                .map(|tt| {
                    MaybeUnloadedTypeHandle::from_token_for_type(
                        assembly,
                        assembly_id,
                        b_assembly,
                        tt,
                        interface_id,
                    )
                })
                .try_collect()?,
            |rt_interface| {
                MethodTable::new(rt_interface, |mt| {
                    self.load_binary_methods(
                        assembly,
                        assembly_id,
                        b_assembly,
                        interface_id,
                        mt,
                        &interface_def.method_table,
                    )
                    .unwrap()
                })
                .as_non_null_ptr()
            },
            self.load_binary_generic_bounds(
                assembly,
                assembly_id,
                b_assembly,
                interface_id,
                &interface_def.generic_bounds,
            )?,
        );

        assert_eq!(assembly.add_type(result), interface_id);
        Ok(())
    }

    fn load_binary_field(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        t_id: u32,
        field: &binary::ty::Field,
    ) -> binary::prelude::BinaryResult<Field> {
        Ok(Field::new(
            widestring::Utf16String::from_str(b_assembly.get_string(field.name)?),
            field.attr,
            MaybeUnloadedTypeHandle::from_token_for_type(
                assembly,
                assembly_id,
                b_assembly,
                &field.ty,
                t_id,
            )?,
        ))
    }

    fn load_binary_methods<T: GetTypeVars + GetAssemblyRef>(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        t_id: u32,
        mt: NonNull<MethodTable<T>>,
        methods: &[binary::ty::Method],
    ) -> binary::prelude::BinaryResult<Vec<Pin<Box<Method<T>>>>> {
        methods
            .iter()
            .map(|method| {
                self.load_binary_method(assembly, assembly_id, b_assembly, t_id, mt, method)
            })
            .try_collect()
    }

    fn load_binary_method<T: GetTypeVars + GetAssemblyRef>(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        t_id: u32,
        mt: NonNull<MethodTable<T>>,
        method: &binary::ty::Method,
    ) -> binary::prelude::BinaryResult<Pin<Box<Method<T>>>> {
        let name = widestring::Utf16String::from_str(b_assembly.get_string(method.name)?);
        let attr = method
            .attr
            .clone()
            .map_types(|tt| {
                MaybeUnloadedTypeHandle::from_token_for_type(
                    assembly,
                    assembly_id,
                    b_assembly,
                    &tt,
                    t_id,
                )
                .map(CachedTypeReference::from)
            })
            .transpose()?;
        Ok(Method::try_new(
            mt,
            name,
            attr,
            method.generic_count_requirement.into(),
            method
                .args
                .iter()
                .map(|param| {
                    MaybeUnloadedTypeHandle::from_token_for_type(
                        assembly,
                        assembly_id,
                        b_assembly,
                        &param.ty,
                        t_id,
                    )
                    .map(|ty| Parameter::new(ty, param.attr))
                })
                .try_collect()?,
            MaybeUnloadedTypeHandle::from_token_for_type(
                assembly,
                assembly_id,
                b_assembly,
                &method.return_type,
                t_id,
            )?
            .into(),
            method.call_convention,
            self.load_binary_generic_bounds(
                assembly,
                assembly_id,
                b_assembly,
                t_id,
                &method.generic_bounds,
            )?,
            method
                .instructions
                .iter()
                .map(|ins| {
                    ins.clone()
                        .map(
                            |s| b_assembly.get_string(s).map(ToOwned::to_owned),
                            |tt| {
                                MaybeUnloadedTypeHandle::from_token_for_type(
                                    assembly,
                                    assembly_id,
                                    b_assembly,
                                    &tt,
                                    t_id,
                                )
                                .map(CachedTypeReference::new)
                            },
                            |tt| {
                                MethodRef::from_token_for_type(
                                    assembly,
                                    assembly_id,
                                    b_assembly,
                                    &tt,
                                    t_id,
                                )
                            },
                            Ok::<_, binary::prelude::Error>,
                        )
                        .transpose::<binary::prelude::Error>()
                })
                .try_collect()?,
            |rt_method| {
                method
                    .exception_table
                    .iter()
                    .map(|entry| {
                        Ok::<ExceptionTableEntry, binary::prelude::Error>(ExceptionTableEntry::new(
                            entry.range,
                            MaybeUnloadedTypeHandle::from_token_for_type(
                                assembly,
                                assembly_id,
                                b_assembly,
                                &entry.exception_type,
                                t_id,
                            )?,
                            entry
                                .filter
                                .map(|(ty, method)| {
                                    MaybeUnloadedTypeHandle::from_token_for_type(
                                        assembly,
                                        assembly_id,
                                        b_assembly,
                                        &ty,
                                        t_id,
                                    )
                                    .and_then(|ty| {
                                        MethodRef::from_token_for_type(
                                            assembly,
                                            assembly_id,
                                            b_assembly,
                                            &method,
                                            t_id,
                                        )
                                        .map(|method| (ty, method))
                                    })
                                })
                                .transpose()?,
                            entry.catch,
                            entry.finally,
                            entry.fault,
                        ))
                    })
                    .try_fold(
                        ExceptionTable::new(NonNull::from_ref(rt_method)),
                        |mut exec, entry| match entry {
                            Ok(entry) => {
                                exec.push(entry);
                                Ok(exec)
                            }
                            Err(err) => Err(err),
                        },
                    )
            },
        )?)
    }

    fn load_binary_generic_bounds(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        t_id: u32,
        generic_bounds: &Option<Vec<binary::ty::GenericBounds>>,
    ) -> binary::prelude::BinaryResult<Option<Vec<GenericBounds>>> {
        generic_bounds
            .as_ref()
            .map(|x| {
                x.iter()
                    .map(|generic_bound| {
                        self.load_binary_generic_bound(
                            assembly,
                            assembly_id,
                            b_assembly,
                            t_id,
                            generic_bound,
                        )
                    })
                    .try_collect()
            })
            .transpose()
    }

    fn load_binary_generic_bound(
        &self,
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        t_id: u32,
        generic_bound: &binary::ty::GenericBounds,
    ) -> binary::prelude::BinaryResult<GenericBounds> {
        let implemented_interfaces = generic_bound
            .implemented_interfaces
            .iter()
            .map(|x| {
                MaybeUnloadedTypeHandle::from_token_for_type(
                    assembly,
                    assembly_id,
                    b_assembly,
                    x,
                    t_id,
                )
            })
            .try_fold(Vec::new(), |mut acc, val| {
                val.map(|val| {
                    acc.push(val);
                    acc
                })
            })?;

        let parent = generic_bound
            .parent
            .as_ref()
            .map(|x| {
                MaybeUnloadedTypeHandle::from_token_for_type(
                    assembly,
                    assembly_id,
                    b_assembly,
                    x,
                    t_id,
                )
            })
            .transpose()?;

        Ok(GenericBounds {
            implemented_interfaces,
            parent,
        })
    }
}

impl MaybeUnloadedTypeHandle {
    fn from_token_for_type(
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        tt: &binary::prelude::TypeToken,
        t_id: u32,
    ) -> binary::prelude::BinaryResult<Self> {
        match tt.ty() {
            binary::prelude::TypeType::TypeDef => {
                if tt.index() < t_id {
                    let th = assembly.get_type_handle(tt.index()).unwrap();
                    Ok(th.into())
                } else {
                    Ok(TypeRef::Index {
                        assembly: AssemblyRef::Id(assembly_id),
                        ind: tt.index(),
                    }
                    .into())
                }
            }
            binary::prelude::TypeType::TypeRef => {
                let actual = b_assembly
                    .type_refs
                    .get(tt.index() as usize)
                    .ok_or(binary::binary_core::Error::IndexOutOfRange)?;

                Ok(TypeRef::Index {
                    assembly: AssemblyRef::Name(StringName::from_string(
                        b_assembly.get_string(*actual.assembly())?.to_owned(),
                    )),
                    ind: *actual.index(),
                }
                .into())
            }
            binary::prelude::TypeType::TypeSpec => {
                let actual = b_assembly
                    .type_specs
                    .get(tt.index() as usize)
                    .ok_or(binary::binary_core::Error::IndexOutOfRange)?;
                let ty = Self::from_token_for_type(
                    assembly,
                    assembly_id,
                    b_assembly,
                    actual.ty(),
                    t_id,
                )?;
                let types = actual
                    .generics()
                    .iter()
                    .map(|x| Self::from_token_for_type(assembly, assembly_id, b_assembly, x, t_id))
                    .try_collect()?;

                Ok(TypeRef::Specific {
                    assembly_and_index: Either::Right(Box::new(ty)),
                    types,
                }
                .into())
            }
            binary::prelude::TypeType::MethodGeneric => todo!(),
            binary::prelude::TypeType::TypeGeneric => Ok(MaybeUnloadedTypeHandle::Loaded(
                TypeHandle::TypeGeneric(tt.index()),
            )),
        }
    }
}

impl MethodRef {
    fn from_token_for_type(
        assembly: &Assembly,
        assembly_id: usize,
        b_assembly: &binary::assembly::Assembly,
        tt: &binary::prelude::MethodToken,
        t_id: u32,
    ) -> binary::prelude::BinaryResult<Self> {
        match tt.ty() {
            binary::prelude::MethodType::Method => Ok(Self::Index(tt.index())),
            binary::prelude::MethodType::MethodSpec => {
                let actual = b_assembly
                    .method_specs
                    .get(tt.index() as usize)
                    .ok_or(binary::binary_core::Error::IndexOutOfRange)?;
                let generics = actual
                    .generics
                    .iter()
                    .map(|x| {
                        MaybeUnloadedTypeHandle::from_token_for_type(
                            assembly,
                            assembly_id,
                            b_assembly,
                            x,
                            t_id,
                        )
                    })
                    .try_collect()?;

                Ok(Self::Specific {
                    index: actual.m,
                    types: generics,
                })
            }
            binary::prelude::MethodType::MethodByRuntime => Ok(Self::Index(tt.index())),
        }
    }
}

impl const From<binary::ty::GenericCountRequirement> for GenericCountRequirement {
    fn from(value: binary::ty::GenericCountRequirement) -> Self {
        match value {
            binary::ty::GenericCountRequirement::AtLeast(range_from) => Self::AtLeast(range_from),
            binary::ty::GenericCountRequirement::NoMoreThan(range_to_inclusive) => {
                Self::NoMoreThan(range_to_inclusive)
            }
            binary::ty::GenericCountRequirement::Exact(val) => Self::Exact(val),
        }
    }
}
