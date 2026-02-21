use global::getset::Getters;

use crate::type_system::type_handle::MaybeUnloadedTypeHandle;

#[derive(Getters)]
pub struct GenericBounds {
    #[allow(dead_code)]
    pub(crate) implemented_interfaces: Vec<MaybeUnloadedTypeHandle>,
    #[allow(dead_code)]
    pub(crate) parent: Option<MaybeUnloadedTypeHandle>,
}
