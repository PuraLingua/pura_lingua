use global::StringName;

use crate::type_system::type_handle::MaybeUnloadedTypeHandle;

#[derive(Clone, PartialEq, Debug)]
pub enum TypeRef {
    Index {
        assembly: StringName,
        ind: u32,
    },
    Specific {
        assembly: StringName,
        ind: u32,
        types: Vec<MaybeUnloadedTypeHandle>,
    },
}
