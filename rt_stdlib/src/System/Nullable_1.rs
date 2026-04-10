use crate::{CoreTypeId, CoreTypeRef};

proc_macros::define_core_struct! {
    #[Public {}] assembly
    System_Nullable_1 1 "System::Nullable`1" =>
    #fields:
    #[Private {}] Inner "_Inner" => CoreTypeRef::Generic(0);

    #methods:
    [] [
        #[Public {Static}] Initialize (
            #[{ByRef}] CoreTypeRef::WithGeneric(
                CoreTypeId::System_Nullable_1,
                vec![CoreTypeRef::Generic(0)],
            )
            #[{}] CoreTypeRef::Generic(0)
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
    ]
}
