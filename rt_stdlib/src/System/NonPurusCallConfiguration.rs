use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_NonPurusCallConfiguration "System::NonPurusCallConfiguration"
    Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields:
    #[Public {}]
    CallConvention "CallConvention" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ReturnType "ReturnType" => CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
    #[Public {}]
    Encoding "Encoding" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ObjectStrategy "ObjectStrategy" => CoreTypeRef::Core(CoreTypeId::System_UInt8);
    #[Public {}]
    ByRefArguments "ByRefArguments" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeRef::Core(CoreTypeId::System_USize),
        ],
    );
    #[Public {}]
    Arguments "Arguments" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Array_1,
        vec![
            CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType),
        ],
    );

    #methods of super::Object::MethodId:
    [
        // Skip sign for default constructor
        #[Public {}] Constructor ".ctor" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::Core(CoreTypeId::System_UInt8)
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![CoreTypeId::System_USize.into()],
            )
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![CoreTypeId::System_NonPurusCallType.into()],
            )
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ] []
}
