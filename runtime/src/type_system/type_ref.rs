use either::Either;

use crate::type_system::type_handle::{IGenericResolver, MaybeUnloadedTypeHandle};

use super::{
    assembly_manager::{AssemblyManager, AssemblyRef},
    type_handle::TypeHandle,
};

#[derive(Clone, PartialEq, Debug)]
pub enum TypeRef {
    Index {
        assembly: AssemblyRef,
        ind: u32,
    },
    Specific {
        assembly_and_index: Either<(AssemblyRef, u32), Box<MaybeUnloadedTypeHandle>>,
        types: Vec<MaybeUnloadedTypeHandle>,
    },
}

impl TypeRef {
    pub fn load_with_generic_resolver<TResolver: IGenericResolver>(
        &self,
        assembly_manager: &AssemblyManager,
        resolver: &TResolver,
    ) -> Option<TypeHandle> {
        match self {
            TypeRef::Index { assembly, ind } => {
                let assembly = assembly_manager.get_assembly_by_ref(assembly).unwrap()?;
                assembly
                    .get_type_handle(*ind)
                    .unwrap()
                    .map(|x| *x)
                    .map(TypeHandle::from)
            }
            TypeRef::Specific {
                assembly_and_index,
                types,
            } => {
                let type_vars: Vec<_> = types
                    .iter()
                    .map(|x| {
                        x.load_with_generic_resolver(assembly_manager, resolver)
                            .and_then(|x| x.get_non_generic_with_generic_resolver(resolver))
                    })
                    .try_collect()?;

                match assembly_and_index {
                    Either::Left((assembly, ind)) => {
                        let assembly = assembly_manager.get_assembly_by_ref(assembly).unwrap()?;
                        let ty = assembly.get_type_handle(*ind).unwrap()?;
                        Some(ty.instantiate(&type_vars).into())
                    }
                    Either::Right(mth) => mth
                        .load_with_generic_resolver(assembly_manager, resolver)
                        .map(|x| match x.as_non_generic() {
                            None => x,
                            Some(x) => x.instantiate(&type_vars).into(),
                        }),
                }
            }
        }
    }
}
