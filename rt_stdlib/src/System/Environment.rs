use proc_macros::define_core_class;

use crate::CoreTypeId;

define_core_class! {
    #[Public {}] assembly
    System_Environment "System::Environment" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:
    #[Public {}] NewLine "NewLine" => CoreTypeId::System_String.into();

    #methods of super::Object::MethodId:
    [] []
}
