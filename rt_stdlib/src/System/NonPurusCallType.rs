use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_class! {
    #[Public {}] assembly
    System_NonPurusCallType "System::NonPurusCallType" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:
    #[Public {}]
    Discriminant "Discriminant" => CoreTypeId::System_UInt8.into();
    #[Public {}]
    Types "Types" => CoreTypeRef::WithGeneric(
        CoreTypeId::System_Nullable_1,
        vec![
            CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![
                    CoreTypeId::System_NonPurusCallType.into(),
                ],
            )
        ],
    );

    #methods of super::Object::MethodId:
    [] [
        #[Public {Static}] CreateVoid () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU8 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI8 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU16 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI16 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU32 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI32 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateU64 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateI64 () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreatePointer () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateString () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
        #[Public {Static}] CreateObject () -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);

        #[Public {Static}] CreateStructure (
            #[{}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Array_1,
                vec![
                    CoreTypeId::System_NonPurusCallType.into(),
                ],
            )
        ) -> CoreTypeRef::Core(CoreTypeId::System_NonPurusCallType);
    ]
}
