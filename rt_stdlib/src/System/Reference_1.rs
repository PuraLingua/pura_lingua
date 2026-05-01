use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Reference_1 1 "System::Reference`1" =>
    #fields:
    #[Public {}]
    Data "Data" => CoreTypeRef::Core(CoreTypeId::System_Pointer);

    #methods:
    [] [
        #[
            Public {Static}
            CoreTypeRef::WithGeneric(CoreTypeId::System_Reference_1, vec![CoreTypeRef::TypeGeneric(0)]), // this
            CoreTypeRef::Core(CoreTypeId::System_USize), // size
            CoreTypeRef::TypeGeneric(0), // result
        ]
        Read (
            #[{ByRef}] CoreTypeRef::WithGeneric(CoreTypeId::System_Reference_1, vec![CoreTypeRef::TypeGeneric(0)])
        ) -> CoreTypeRef::TypeGeneric(0);
        #[
            Public {Static}
            CoreTypeRef::WithGeneric(CoreTypeId::System_Reference_1, vec![CoreTypeRef::TypeGeneric(0)]), // this
            CoreTypeRef::TypeGeneric(0), // data
            CoreTypeRef::Core(CoreTypeId::System_USize), // size
        ]
        Write (
            #[{ByRef}] CoreTypeRef::WithGeneric(CoreTypeId::System_Reference_1, vec![CoreTypeRef::TypeGeneric(0)])
            #[{}] CoreTypeRef::TypeGeneric(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ]
}
