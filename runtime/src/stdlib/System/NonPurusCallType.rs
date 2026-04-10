use global::non_purus_call_configuration::NonPurusCallType;
use stdlib_header::{CoreTypeId, System::NonPurusCallType::FieldId};

use crate::{
    stdlib::System::{_define_class, common_new_method, default_sctor},
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

#[inline(always)]
fn alloc_this(cpu: &mut CPU) -> ManagedReference<Class> {
    ManagedReference::common_alloc(
        cpu,
        unsafe {
            cpu.vm_ref()
                .assembly_manager()
                .get_core_type(CoreTypeId::System_NonPurusCallType)
                .unwrap_class()
                .as_ref()
                .method_table
        },
        false,
    )
}

macro define_common(
    $Creator:ident $ToBeCreated:ident
) {
    pub extern "system" fn $Creator(
        cpu: &mut $crate::virtual_machine::cpu::CPU,
        _: &$crate::type_system::method::Method<$crate::type_system::class::Class>,
    ) -> $crate::value::managed_reference::ManagedReference<Class> {
        let mut managed = alloc_this(cpu);
        assert!(
            managed
                .const_access_mut::<$crate::value::managed_reference::FieldAccessor<Class>>()
                .write_typed_field(
                    ::stdlib_header::System::NonPurusCallType::FieldId::Discriminant as _,
                    Default::default(),
                    ::global::non_purus_call_configuration::NonPurusCallType::$ToBeCreated
                        .discriminant(),
                ),
            "Failed to access field `Discriminant`",
        );
        managed
    }
}

define_common!(CreateVoid Void);

define_common!(CreateU8 U8);
define_common!(CreateI8 I8);

define_common!(CreateU16 U16);
define_common!(CreateI16 I16);

define_common!(CreateU32 U32);
define_common!(CreateI32 I32);

define_common!(CreateU64 U64);
define_common!(CreateI64 I64);

define_common!(CreatePointer Pointer);
define_common!(CreateString String);
define_common!(CreateObject Object);

pub extern "system" fn CreateStructure(
    cpu: &mut CPU,
    _: &Method<Class>,
    fields: ManagedReference<Class>,
) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::STRUCTURE_DISCRIMINANT,
            )
    );
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(FieldId::Types as _, Default::default(), fields,)
    );
    managed
}

macro make_common($TMethodId:ident $mt:ident $x:ident) {
    super::common_new_method!(
        $mt
        $TMethodId
        ${concat(Create, $x)}
        ${concat(Create, $x)}
    )
}

_define_class!(
    fn load(assembly, mt, method_info)
    NonPurusCallType
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);

    CreateVoid => make_common!(TStaticMethodId mt Void);

    CreateU8 => make_common!(TStaticMethodId mt U8);
    CreateI8 => make_common!(TStaticMethodId mt I8);

    CreateU16 => make_common!(TStaticMethodId mt U16);
    CreateI16 => make_common!(TStaticMethodId mt I16);

    CreateU32 => make_common!(TStaticMethodId mt U32);
    CreateI32 => make_common!(TStaticMethodId mt I32);

    CreateU64 => make_common!(TStaticMethodId mt U64);
    CreateI64 => make_common!(TStaticMethodId mt I64);

    CreatePointer => make_common!(TStaticMethodId mt Pointer);

    CreateString => make_common!(TStaticMethodId mt String);
    CreateObject => make_common!(TStaticMethodId mt Object);
    CreateStructure => common_new_method!(mt TStaticMethodId CreateStructure CreateStructure);
);
