use global::non_purus_call_configuration::NonPurusCallType;
use stdlib_header::CoreTypeId;

use crate::{
    stdlib::System_NonPurusCallType_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference},
    virtual_machine::cpu::CPU,
};

#[inline(always)]
fn alloc_this(cpu: &CPU) -> ManagedReference<Class> {
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

pub extern "system" fn CreateVoid(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::Void.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateU8(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::U8.discriminant(),
            )
    );
    managed
}
pub extern "system" fn CreateI8(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::I8.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateU16(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::U16.discriminant(),
            )
    );
    managed
}
pub extern "system" fn CreateI16(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::I16.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateU32(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::U32.discriminant(),
            )
    );
    managed
}
pub extern "system" fn CreateI32(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::I32.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateU64(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                dbg!(NonPurusCallType::U64.discriminant()),
            )
    );
    managed
}
pub extern "system" fn CreateI64(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::I64.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreatePointer(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::Pointer.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateString(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::String.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateObject(cpu: &CPU, _: &Method<Class>) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::Object.discriminant(),
            )
    );
    managed
}

pub extern "system" fn CreateStructure(
    cpu: &CPU,
    _: &Method<Class>,
    fields: ManagedReference<Class>,
) -> ManagedReference<Class> {
    let mut managed = alloc_this(cpu);
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Discriminant as _,
                Default::default(),
                NonPurusCallType::STRUCTURE_DISCRIMINANT,
            )
    );
    assert!(
        managed
            .const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field(
                System_NonPurusCallType_FieldId::Types as _,
                Default::default(),
                fields,
            )
    );
    managed
}
