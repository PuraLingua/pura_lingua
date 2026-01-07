use std::io::Write;
use std::sync::Arc;

use enumflags2::make_bitflags;
use global::StringMethodReference;
use global::attrs::MethodAttr;
use global::attrs::MethodImplementationFlags;
use global::attrs::ParameterAttr;
use global::attrs::ParameterImplementationFlags;
use global::attrs::{
    ClassImplementationFlags, FieldAttr, FieldImplementationFlags, TypeAttr, TypeSpecificAttr,
    Visibility,
};
use global::instruction::Instruction;
use global::instruction::StringInstruction;
use global::item_token::ItemToken;
use global::item_token::ItemTokenBuilder;
use global::item_token::ItemType;
use global::string_method_ref_ret;
use global::{StringName, StringTypeReference, indexmap, string_name};

use crate::assembly::Assembly;
use crate::method::Method;
use crate::ty::class::ClassDef;
use crate::ty::{GenericBinding, TypeDef, class};

#[test]
fn test_emit_get() -> global::Result<()> {
    const TEST_CLASS_NAME: StringTypeReference =
        StringTypeReference::make_static_single("Test", "Test.Test");
    let mut assem = Assembly::default();
    *assem.name_mut() = StringName::from_static_str("Test");
    assem.type_defs_mut().insert(
        string_name!("Test.Test"),
        TypeDef::Class(ClassDef::new(
            Some(StringTypeReference::System_Object),
            indexmap! {
                string_name!("@T") => GenericBinding::new(
                    vec![StringTypeReference::from_string_repr("[!]System.IDisposable")?],
                    Some(StringTypeReference::from_string_repr("[!]System.Array`1[@T:[!]System.Object]")?)
                ),
                string_name!("@U") => GenericBinding::new(
                    vec![],
                    None,
                )
            },
            global::attr!(
                class Public {}
            ),
            Vec::new(),
            string_name!("Test.Test"),
            indexmap! {
                StringMethodReference::from_string_repr("PrintStaticsAndGenericType()")? =>
                Method::new(
                    StringMethodReference::from_string_repr("PrintStaticsAndGenericType()")?,
                    global::attr!(
                        method Public {Static}
                        StringTypeReference::System_String,
                        StringTypeReference::System_Void,
                    ),
                    Vec::new(),
                    vec![
                        Instruction::LoadStatic {
                            register_addr: 0,
                            ty: ItemTokenBuilder::new()
                                    .with_ty(ItemType::TypeDef)
                                    .with_i(0)
                                    .build(),
                            name: ItemTokenBuilder::new()
                                    .with_ty(ItemType::Field)
                                    .with_i(0)
                                    .build(),
                        },
                        Instruction::StaticCall {
                            ty: StringTypeReference::core_static_single_type("System.Console"),
                            method: StringMethodReference::Single {
                                interface: None,
                                name: string_name!("WriteLine"),
                                parameters: Arc::new([
                                    StringTypeReference::System_String,
                                ]),
                            },
                            args: vec![0],
                            ret_at: 1,
                        },
                    ],
                    StringTypeReference::System_Void,
                    vec![],
                    Default::default(),
                ),
                global::ENTRY_POINT_REF.clone()
                => Method::new(
                    global::ENTRY_POINT_REF.clone(),
                    global::attr!(
                        method Public {Static}
                        StringTypeReference::System_Void,
                        StringTypeReference::System_UInt64,
                    ),
                    Vec::new(),
                    vec![
                        StringInstruction::StaticCall {
                            ty: TEST_CLASS_NAME,
                            method: string_method_ref_ret!("PrintStaticsAndGenericType()"),
                            args: vec![],
                            ret_at: 1,
                        },

                        StringInstruction::Load_u64 {
                            register_addr: 1,
                            val: 0,
                        },
                        StringInstruction::ReturnVal {
                            register_addr: 1,
                        }
                    ],
                    StringTypeReference::core_static_single_type("System.Void"),
                    vec![
                        (
                            StringTypeReference::WithGeneric {
                                assem: string_name!("!"),
                                ty: string_name!("System.Array`1"),
                                type_vars: Arc::new(indexmap! {
                                    string_name!("@T") => StringTypeReference::System_String.into(),
                                })
                            },
                            global::attr!(
                                parameter {}
                            ),
                        )
                    ],
                    Default::default(),
                ),
                StringMethodReference::STATIC_CTOR_REF => Method::new(
                    StringMethodReference::STATIC_CTOR_REF,
                    MethodAttr::new(
                        Visibility::Public,
                        make_bitflags!(MethodImplementationFlags::{Static}),
                        vec![
                            StringTypeReference::System_UInt64,
                            StringTypeReference::System_String,
                        ],
                    ),
                    Vec::new(),
                    vec![
                        StringInstruction::Load_u64 {
                            register_addr: 0,
                            val: 10,
                        },
                        StringInstruction::InstanceCall {
                            val: 0,
                            method: StringMethodReference::from_string_repr("ToString()")?,
                            this: StringTypeReference::System_UInt64,
                            args: vec![],
                            ret_at: 1,
                        },
                        StringInstruction::SetStaticField {
                            val_addr: 1,
                            ty: StringTypeReference::Single {
                                assem: string_name!("Test"),
                                ty: string_name!("Test.Test")
                            },
                            field: string_name!("__test"),
                        },
                    ],
                    StringTypeReference::core_static_single_type("System.Void"),
                    vec![],
                    Default::default(),
                ),
            },
            indexmap! {
                StringName::from_static_str("__test") => class::Field {
                    name: StringName::from_static_str("__test"),
                    attr: FieldAttr::new(Visibility::Public, make_bitflags!(FieldImplementationFlags::{Static})),
                    custom_attributes: Vec::new(),
                    ty: StringTypeReference::make_static_single("!", "System.String"),
                }
            }
        )),
    );
    let b = assem.to_file_bytes()?;
    print!("Out to file?[Y/n] ");
    std::io::stdout().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    if s.to_ascii_lowercase().starts_with("y") {
        std::fs::write("./test_data/test.plb", &b)?;
        assert_eq!(std::fs::read("./test_data/test.plb")?, b);
    }
    let assem_gotten = Assembly::from_bytes(b)?;
    dbg!(&assem_gotten);
    Ok(())
}

#[test]
fn test_get_only() -> global::Result<()> {
    let assem = Assembly::from_file("./test_data/test.plb")?;
    dbg!(&assem);
    Ok(())
}

#[test]
fn emit_test_exception() -> global::Result<()> {
    const TEST_CLASS_NAME: StringTypeReference =
        StringTypeReference::make_static_single("Test", "Test.Test");
    let mut assem = Assembly::default();
    *assem.name_mut() = StringName::from_static_str("Test");
    assem.type_defs_mut().insert(
        string_name!("Test.Test"),
        TypeDef::Class(ClassDef::new(
            Some(StringTypeReference::System_Object),
            indexmap! {
                string_name!("@T") => GenericBinding::new(
                    vec![StringTypeReference::from_string_repr("[!]System.IDisposable")?],
                    Some(StringTypeReference::from_string_repr("[!]System.Array`1[@T:[!]System.Object]")?)
                ),
                string_name!("@U") => GenericBinding::new(
                    vec![],
                    None,
                )
            },
            global::attr!(
                class Public {}
            ),
            Vec::new(),
            string_name!("Test.Test"),
            indexmap! {
                global::ENTRY_POINT_REF.clone()
                => Method::new(
                    global::ENTRY_POINT_REF.clone(),
                    global::attr!(
                        method Public {Static}
                        StringTypeReference::System_Void,
                        StringTypeReference::System_UInt64,
                        StringTypeReference::System_Exception,
                        StringTypeReference::System_String,
                    ),
                    Vec::new(),
                    vec![
                        StringInstruction::NewObject {
                            ty: StringTypeReference::System_Exception,
                            ctor_name: string_method_ref_ret!(".ctor()"),
                            args: vec![],
                            register_addr: 2,
                        },
                        StringInstruction::InstanceCall {
                            val: 2,
                            this: StringTypeReference::System_Exception,
                            method: string_method_ref_ret!("ToString()"),
                            args: vec![],
                            ret_at: 3,
                        },
                        StringInstruction::StaticCall {
                            ty: StringTypeReference::core_static_single_type("System.Console"),
                            method: StringMethodReference::Single {
                                interface: None,
                                name: string_name!("WriteLine"),
                                parameters: Arc::new([
                                    StringTypeReference::System_String,
                                ]),
                            },
                            args: vec![3],
                            ret_at: 0,
                        },
                        StringInstruction::Throw { exception_addr: 2 },

                        StringInstruction::Load_u64 {
                            register_addr: 1,
                            val: 0,
                        },
                        StringInstruction::ReturnVal {
                            register_addr: 1,
                        }
                    ],
                    StringTypeReference::System_Void,
                    vec![
                        (
                            StringTypeReference::WithGeneric {
                                assem: string_name!("!"),
                                ty: string_name!("System.Array`1"),
                                type_vars: Arc::new(indexmap! {
                                    string_name!("@T") => StringTypeReference::System_String.into(),
                                })
                            },
                            global::attr!(
                                parameter {}
                            ),
                        )
                    ],
                    Default::default(),
                ),
            },
            indexmap! {}
        )),
    );
    let b = assem.to_file_bytes()?;
    std::fs::write("./test_data/test_exception.plb", &b)?;
    let assem_gotten = Assembly::from_bytes(b)?;
    dbg!(&assem_gotten);
    Ok(())
}
