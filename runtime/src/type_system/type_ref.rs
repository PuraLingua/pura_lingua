use either::Either;

use crate::type_system::type_handle::MaybeUnloadedTypeHandle;

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
    pub fn load(&self, assembly_manager: &AssemblyManager) -> Option<TypeHandle> {
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
            } => match assembly_and_index {
                Either::Left((assembly, ind)) => {
                    let assembly = assembly_manager.get_assembly_by_ref(assembly).unwrap()?;
                    let ty = assembly.get_type_handle(*ind).unwrap()?;
                    Some(ty.instantiate(types).into())
                }
                Either::Right(mth) => {
                    mth.load(assembly_manager)
                        .map(|x| match x.as_non_generic() {
                            None => x,
                            Some(x) => x.instantiate(types).into(),
                        })
                }
            },
        }
    }
}
