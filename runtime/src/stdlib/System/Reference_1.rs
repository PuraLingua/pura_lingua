use global::{
    attrs::CallConvention,
    instruction::{
        CommonReadPointerTo, CommonWritePointer, IRegisterAddr as _, Instruction, Instruction_Load,
        LoadContent, ShortRegisterAddr,
    },
};

use crate::{
    stdlib::System::{_define_struct, default_sctor, map_method_attr, map_parameters},
    type_system::{
        generics::GenericCountRequirement,
        method::{ExceptionTable, Method},
        type_handle::TypeHandle,
    },
};

_define_struct!(
    fn load(assembly, mt, method_info)
    Reference_1
#methods(TMethodId):
#static_methods(TStaticMethodId):
    StaticConstructor => default_sctor!(mt TStaticMethodId);
    Read => Method::new(
        mt,
        method_info.name,
        map_method_attr(method_info.attr),
        GenericCountRequirement::default(),
        map_parameters(method_info.args),
        method_info.return_type.into(),
        CallConvention::PlatformDefault,
        None,
        vec![
            Instruction::SLoad(Instruction_Load {
                addr: ShortRegisterAddr::new(0), // this
                content: LoadContent::ArgValue(0),
            }),
            Instruction::SLoad(Instruction_Load {
                addr: ShortRegisterAddr::new(1), // size
                content: LoadContent::TypeValueSize(TypeHandle::TypeGeneric(0).into()),
            }),
            Instruction::SReadPointerTo(CommonReadPointerTo {
                ptr: ShortRegisterAddr::new(0),
                size: ShortRegisterAddr::new(1),
                destination: ShortRegisterAddr::new(2),
            }),
            Instruction::SReturnVal {
                register_addr: ShortRegisterAddr::new(2),
            },
        ],
        ExceptionTable::gen_new(),
    );
    Write => Method::new(
        mt,
        method_info.name,
        map_method_attr(method_info.attr),
        GenericCountRequirement::default(),
        map_parameters(method_info.args),
        method_info.return_type.into(),
        CallConvention::PlatformDefault,
        None,
        vec![
            Instruction::SLoad(Instruction_Load {
                addr: ShortRegisterAddr::new(0), // this
                content: LoadContent::ArgValue(0),
            }),
            Instruction::SLoad(Instruction_Load {
                addr: ShortRegisterAddr::new(1), // data
                content: LoadContent::Arg(1),
            }),
            Instruction::SLoad(Instruction_Load {
                addr: ShortRegisterAddr::new(2), // size
                content: LoadContent::TypeValueSize(TypeHandle::TypeGeneric(0).into()),
            }),
            Instruction::SWritePointer(CommonWritePointer {
                source: ShortRegisterAddr::new(1),
                size: ShortRegisterAddr::new(2),
                ptr: ShortRegisterAddr::new(0),
            })
        ],
        ExceptionTable::gen_new(),
    );
);
