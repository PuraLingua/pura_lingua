use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_NullReferenceException Some((CoreTypeId::System_Exception.into(), vec![])) =>
    #fields of super::Exception::FieldId:
    #[Public {}] Name "_Name" => CoreTypeId::System_String.into();

    #methods of super::Exception::MethodId:
    [
        #[Public {}] Constructor ".ctor" () -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {}] Constructor_String ".ctor([!]System::String)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_String)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
