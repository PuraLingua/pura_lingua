use std::io::Write;

use binary_core::section::Section;
use global::{
    attrs::CallConvention,
    instruction::{Instruction, RegisterAddr},
};
use stdlib_header::{
    CoreTypeId,
    definitions::{
        System_Exception_MethodId, System_Object_MethodId, System_UInt64_StaticMethodId,
    },
};

use crate::{
    assembly::ExtraHeader,
    prelude::{Assembly, MethodTokenBuilder, MethodType, TypeTokenBuilder, TypeType},
    ty::{ClassDef, Method, TypeDef, TypeRef},
};

macro core_type_ref($s_section:expr => $i:ident) {
    TypeRef {
        assembly: $s_section.as_string_section_mut().add_string("!"),
        index: CoreTypeId::$i as _,
    }
}

macro impl_into_method_token($t:ty) {
    impl From<$t> for $crate::item_token::MethodToken {
        fn from(value: $t) -> Self {
            $crate::item_token::MethodTokenBuilder::new()
                .with_ty($crate::item_token::MethodType::Method)
                .with_index(value as _)
                .build()
        }
    }
}

impl_into_method_token!(System_UInt64_StaticMethodId);
impl_into_method_token!(System_Exception_MethodId);

#[test]
fn emit_test_normal_f() -> binary_core::BinaryResult<()> {
    let mut section = Section::new();
    let assembly = Assembly {
        extra_header: ExtraHeader {
            name: section.as_string_section_mut().add_string("TestNormalF"),
        },
        custom_attributes: Vec::new(),
        type_refs: vec![
            core_type_ref!(section => System_Object),
            core_type_ref!(section => System_Void),
            core_type_ref!(section => System_UInt64),
            core_type_ref!(section => System_String),
            core_type_ref!(section => System_Exception),
            TypeRef {
                assembly: section.as_string_section_mut().add_string("TestNormalF"),
                index: 0,
            },
        ],
        type_specs: Vec::new(),
        method_specs: Vec::new(),
        type_defs: vec![TypeDef::Class(ClassDef {
            main: None,
            name: section
                .as_string_section_mut()
                .add_string("TestNormalF::Test"),
            attr: global::attr!(class Public {}),
            parent: Some(
                TypeTokenBuilder::new()
                    .with_ty(TypeType::TypeRef)
                    .with_index(0)
                    .build(),
            ),
            method_table: vec![
                Method {
                    name: section.as_string_section_mut().add_string("F1"),
                    attr: global::attr!(
                        method Public {Static}
                        TypeTokenBuilder::new()
                            .with_ty(TypeType::TypeRef)
                            .with_index(2)
                            .build(),
                        TypeTokenBuilder::new()
                            .with_ty(TypeType::TypeRef)
                            .with_index(3)
                            .build(),
                        TypeTokenBuilder::new()
                            .with_ty(TypeType::TypeRef)
                            .with_index(4)
                            .build(),
                    ),
                    args: vec![],
                    return_type: TypeTokenBuilder::new()
                        .with_ty(TypeType::TypeRef)
                        .with_index(1)
                        .build(),
                    call_convention: CallConvention::PlatformDefault,
                    generic_bounds: None,
                    instructions: vec![
                        Instruction::Load_u64 {
                            register_addr: RegisterAddr::new(0),
                            val: 10,
                        },
                        Instruction::StaticCall {
                            ty: TypeTokenBuilder::new()
                                .with_ty(TypeType::TypeRef)
                                .with_index(2)
                                .build(),
                            method: System_UInt64_StaticMethodId::ToString.into(),
                            args: vec![RegisterAddr::new(0)],
                            ret_at: RegisterAddr::new(1),
                        },
                        Instruction::NewObject {
                            ty: TypeTokenBuilder::new()
                                .with_ty(TypeType::TypeRef)
                                .with_index(4)
                                .build(),
                            ctor_name: System_Exception_MethodId::Constructor_String.into(),
                            args: vec![RegisterAddr::new(1)],
                            register_addr: RegisterAddr::new(2),
                        },
                        Instruction::Throw {
                            exception_addr: RegisterAddr::new(2),
                        },
                        Instruction::Load_u64 {
                            register_addr: RegisterAddr::new(0),
                            val: 5,
                        }, // Unreachable
                    ],
                },
                Method {
                    name: section.as_string_section_mut().add_string("F2"),
                    attr: global::attr!(
                        method Public {Static}
                        TypeTokenBuilder::new()
                            .with_ty(TypeType::TypeRef)
                            .with_index(1)
                            .build(),
                    ),
                    args: vec![],
                    return_type: TypeTokenBuilder::new()
                        .with_ty(TypeType::TypeRef)
                        .with_index(1)
                        .build(),
                    call_convention: CallConvention::PlatformDefault,
                    generic_bounds: None,
                    instructions: vec![Instruction::StaticCall {
                        ty: TypeTokenBuilder::new()
                            .with_ty(TypeType::TypeDef)
                            .with_index(0)
                            .build(),
                        method: MethodTokenBuilder::new()
                            .with_ty(MethodType::Method)
                            .with_index(System_Object_MethodId::__END as u32)
                            .build(),
                        args: vec![],
                        ret_at: RegisterAddr::new(1),
                    }],
                },
                // Statics
                Method {
                    name: section.as_string_section_mut().add_string(".sctor"),
                    attr: global::attr!(
                        method Public {Static}
                    ),
                    args: Vec::new(),
                    return_type: TypeTokenBuilder::new()
                        .with_ty(TypeType::TypeRef)
                        .with_index(1)
                        .build(),
                    call_convention: CallConvention::PlatformDefault,
                    generic_bounds: None,
                    instructions: Vec::new(),
                },
            ],
            fields: Vec::new(),
            sctor: None,
            generic_bounds: None,
        })],

        string_section: section,
    };
    dbg!(&assembly);

    let file = assembly.into_file()?;
    let mut answer = String::new();
    std::io::stdout().write_all("Should emit to file[Y/N]:".as_bytes())?;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut answer)?;
    answer.make_ascii_lowercase();
    if answer.contains("y") {
        let mut file_out = std::fs::File::create("../TestData/TestNormalF.plb")?;
        file.write_to(&mut file_out)?;
    }

    Ok(())
}
