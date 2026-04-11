use proc_macros::define_core_class;

use crate::{CoreTypeId, CoreTypeRef};

define_core_class! {
    #[Public {}] assembly
    System_RuntimeBasic "System::RuntimeBasic" Some((CoreTypeId::System_Object.into(), vec![])) =>
    #fields of super::Object::FieldId:
    #[Public {}] NewLine "NewLine" => CoreTypeId::System_String.into();

    #methods of super::Object::MethodId:
    []
    [
        /* #region Allocation */
        // They do [de]allocation with the [`std::alloc::Global`] allocator
        #[Public {Static}] Global_Allocate "Global_Allocate([!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[Public {Static}] Global_AllocateZeroed "Global_AllocateZeroed([!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Pointer);
        #[Public {Static}] Global_Deallocate "Global_Deallocate([!]System::Pointer,[!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Pointer) // pointer
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {Static}] Global_Grow "Global_Grow([!]System::Pointer,[!]System::USize,[!]System::USize,[!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Pointer) // pointer

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_align

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {Static}] Global_GrowZeroed "Global_GrowZeroed([!]System::Pointer,[!]System::USize,[!]System::USize,[!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Pointer) // pointer

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_align

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        #[Public {Static}] Global_Shrink "Global_Shrink([!]System::Pointer,[!]System::USize,[!]System::USize,[!]System::USize,[!]System::USize)" (
            #[{}] CoreTypeRef::Core(CoreTypeId::System_Pointer) // pointer

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // old_align

            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // size
            #[{}] CoreTypeRef::Core(CoreTypeId::System_USize) // align
        ) -> CoreTypeRef::Core(CoreTypeId::System_Void);
        /* #endregion */
    ]
}
