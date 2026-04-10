use proc_macros::define_core_class;

use crate::{CoreTypeId, CoreTypeRef};

define_core_class! {
    #[Public {}] assembly
    System_Exception "System::Exception" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:
    #[Public {}] Message "_message" => CoreTypeId::System_String.into();
    #[Public {}] Inner "_innerException" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Nullable_1,
        vec![
            CoreTypeId::System_Exception.into(),
        ],
    );
    #[Public {}] StackTrace "_stackTrace" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeId::System_String.into(),
        ],
    );

    #methods of super::Object::MethodId:
    [
        #[override Some(super::Object::MethodId::ToString as _) Public {}]
        ToString () -> CoreTypeRef::Core(CoreTypeId::System_String);
        #[Public {HideWhenCapturing}] Constructor_String ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
