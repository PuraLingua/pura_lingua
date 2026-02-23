use std::ptr::NonNull;

use either::Either;
use global::StringName;

use crate::type_system::{
    assembly::Assembly,
    assembly_manager::AssemblyRef,
    class::Class,
    field::Field,
    generics::GenericBounds,
    get_traits::{GetAssemblyRef, GetTypeVars},
    method::{Method, MethodRef, Parameter},
    method_table::MethodTable,
    r#struct::Struct,
    type_handle::{MaybeUnloadedTypeHandle, NonGenericTypeHandle, TypeHandle},
    type_ref::TypeRef,
};

use super::AssemblyManager;

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
            loaded_ids.push(self.load_binary(dbg!(b_assembly))?);
        }

        for (b_assembly_id, loaded_id) in loaded_ids.into_iter().enumerate() {
            let assembly = self.get_assembly(loaded_id).unwrap().unwrap();
            let b_assembly = &binaries[b_assembly_id];
            let types = assembly.types.read().unwrap();
            for (t_id, ty) in types.iter().enumerate() {
                match ty {
                    &NonGenericTypeHandle::Class(mut class) => {
                        if unsafe { class.as_ref() }.load_state == TypeLoadState::Finished {
                            continue;
                        }
                        let Some(parent) = (unsafe { &mut class.as_mut().m_parent }) else {
                            continue;
                        };
                        let parent_type_ref = unsafe { Box::from_non_null(parent.unloaded) };
                        parent.loaded = parent_type_ref
                            .load(self)
                            .unwrap()
                            .into_non_generic()
                            .unwrap()
                            .unwrap_class();

                        unsafe {
                            class.as_mut().method_table = MethodTable::new(class, |mt| {
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
                            class.as_mut().rediscover_sctor(
                                b_assembly.type_defs[t_id].unwrap_class_ref().sctor,
                            );
                        }

                        unsafe {
                            class.as_mut().load_state = TypeLoadState::Finished;
                        }
                    }
                    NonGenericTypeHandle::Struct(_) => {}
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
        let id = self.add_assembly(Box::new(Assembly::new(self, name.to_owned(), false)));
        let assembly = self.get_assembly(id).unwrap().unwrap();
        for (type_id, type_def) in binary.type_defs.iter().enumerate() {
            match type_def {
                binary::ty::TypeDef::Class(class_def) => {
                    self.load_binary_class(&assembly, id, binary, class_def, type_id as u32)?;
                }
                binary::ty::TypeDef::Struct(struct_def) => {
                    self.load_binary_struct(&assembly, id, binary, struct_def, type_id as u32)?;
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
        let mut parent_loaded = false;
        let result = Class::new_for_binary(
            NonNull::from_ref(assembly),
            name.to_owned(),
            class_def.attr,
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
                                    parent_loaded = true;
                                    Ok(Either::Left(class))
                                }
                                Some(_) => Err(binary::prelude::Error::WrongParentType),
                            }
                        }
                        MaybeUnloadedTypeHandle::Unloaded(type_ref) => Ok(Either::Right(type_ref)),
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
            self.load_binary_generic_bounds(
                assembly,
                assembly_id,
                b_assembly,
                class_id,
                &class_def.generic_bounds,
            )?,
        );

        assert_eq!(assembly.add_type(result.as_non_null_ptr()), class_id);
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
            name.to_owned(),
            struct_def.attr,
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

        assert_eq!(assembly.add_type(result.as_non_null_ptr()), struct_id);
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
            b_assembly.get_string(field.name)?.to_owned(),
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
    ) -> binary::prelude::BinaryResult<Vec<Box<Method<T>>>> {
        methods
            .iter()
            .map(|method| {
                dbg!();
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
    ) -> binary::prelude::BinaryResult<Box<Method<T>>> {
        let name = b_assembly.get_string(method.name)?.to_owned();
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
            })
            .transpose()?;
        Ok(Box::new(Method::new(
            mt,
            name,
            attr,
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
            )?,
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
                        .map_types(
                            |s| b_assembly.get_string(s).map(ToOwned::to_owned),
                            |tt| {
                                MaybeUnloadedTypeHandle::from_token_for_type(
                                    assembly,
                                    assembly_id,
                                    b_assembly,
                                    &tt,
                                    t_id,
                                )
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
        )))
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
                    let th = assembly.get_type_handle(tt.index()).unwrap().unwrap();
                    Ok((*th).into())
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
            binary::prelude::TypeType::Generic => Ok(MaybeUnloadedTypeHandle::Loaded(
                TypeHandle::Generic(tt.index()),
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
