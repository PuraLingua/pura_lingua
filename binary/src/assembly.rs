use std::{
    io::{Cursor, Seek, Write},
    path::Path,
};

use binary_core::{
    Error,
    file::{File, FileBuilder, PredefinedSectionId},
    section::{Section, SectionBuilder},
    traits::{ReadFromSection as _, StringRef, WriteToSection},
};
use derive_more::Debug;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::{
    custom_attribute::CustomAttribute,
    ty::{MethodSpec, TypeDef, TypeRef, TypeSpec},
};

#[derive(Debug)]
pub struct Assembly<'a> {
    #[debug(
        "ExtraHeader {{ name: {} }}",
        string_section
            .as_string_section()
            .get_string(extra_header.name)
            .unwrap()
    )]
    pub extra_header: ExtraHeader,
    #[debug(skip)]
    pub string_section: &'a Section,
    pub custom_attributes: Vec<CustomAttribute>,
    pub type_refs: Vec<TypeRef>,
    pub type_specs: Vec<TypeSpec>,
    pub method_specs: Vec<MethodSpec>,
    pub type_defs: Vec<TypeDef>,
}

#[derive(Debug)]
pub struct AssemblyBuilder {
    #[debug(
        "ExtraHeader {{ name: {} }}",
        string_section
            .as_string_section()
            .get_string(extra_header.name)
            .unwrap()
    )]
    pub extra_header: ExtraHeader,
    #[debug(skip)]
    pub string_section: SectionBuilder,
    pub custom_attributes: Vec<CustomAttribute>,
    pub type_refs: Vec<TypeRef>,
    pub type_specs: Vec<TypeSpec>,
    pub method_specs: Vec<MethodSpec>,
    pub type_defs: Vec<TypeDef>,
}

#[derive(ReadFromSection, WriteToSection, Debug, Clone, Copy)]
pub struct ExtraHeader {
    pub name: StringRef,
}

#[repr(usize)]
enum AssemblySectionId {
    ExtraHeaderId = PredefinedSectionId::FirstNonStandard as usize,
    CustomAttributes,
    TypeRefs,
    TypeSpecs,
    MethodSpecs,
    TypeDefs,
}

impl<'a> Assembly<'a> {
    pub fn from_builder(assem: &'a AssemblyBuilder) -> Self {
        Self {
            extra_header: assem.extra_header,
            string_section: &assem.string_section,
            custom_attributes: assem.custom_attributes.clone(),
            type_refs: assem.type_refs.clone(),
            type_specs: assem.type_specs.clone(),
            method_specs: assem.method_specs.clone(),
            type_defs: assem.type_defs.clone(),
        }
    }
    pub fn from_file(file: File<'a>) -> binary_core::BinaryResult<Self> {
        let string_section = file
            .get_section(PredefinedSectionId::String as usize)
            .unwrap();
        let extra_header_section = file
            .get_section(AssemblySectionId::ExtraHeaderId as _)
            .ok_or(Error::UnknownSection(AssemblySectionId::ExtraHeaderId as _))?;
        let extra_header = ExtraHeader::read_from_section(&mut Cursor::new(extra_header_section))?;
        let custom_attributes =
            file.read_all::<CustomAttribute>(AssemblySectionId::CustomAttributes as _)?;
        let type_refs = file.read_all::<TypeRef>(AssemblySectionId::TypeRefs as _)?;
        let type_specs = file.read_all::<TypeSpec>(AssemblySectionId::TypeSpecs as _)?;
        let method_specs = file.read_all::<MethodSpec>(AssemblySectionId::MethodSpecs as _)?;
        let type_defs = file.read_all::<TypeDef>(AssemblySectionId::TypeDefs as _)?;

        Ok(Self {
            string_section: string_section,
            extra_header,
            custom_attributes,
            type_refs,
            type_specs,
            method_specs,
            type_defs,
        })
    }
    pub fn from_bytes(bytes: &'a [u8]) -> binary_core::BinaryResult<Self> {
        File::from_bytes(bytes).and_then(|file| Assembly::from_file(file))
    }
    // pub fn from_path<P: AsRef<Path>>(p: P) -> binary_core::BinaryResult<(Assembly, Vec<u8>)> {
    //     let bytes = std::fs::read(p)?;
    //     let this = Assembly::from_bytes(&bytes)?;
    //     Ok((this, bytes))
    // }
    pub fn get_string(&self, string_ref: StringRef) -> binary_core::BinaryResult<&str> {
        self.string_section
            .as_string_section()
            .get_string(string_ref)
            .ok_or_else(|| Error::UnknownStringRef(string_ref))
    }
}

impl AssemblyBuilder {
    pub fn new(name: &str) -> Self {
        let mut string_section = SectionBuilder::new();
        Self {
            extra_header: ExtraHeader {
                name: string_section.as_string_section_mut().add_string(name),
            },
            string_section,
            custom_attributes: Vec::new(),
            type_refs: Vec::new(),
            type_specs: Vec::new(),
            method_specs: Vec::new(),
            type_defs: Vec::new(),
        }
    }
    pub fn from_file(file: FileBuilder) -> binary_core::BinaryResult<Self> {
        let string_section = file
            .get_predefined_section(PredefinedSectionId::String)
            .unwrap();
        let extra_header_section = file
            .get_section(AssemblySectionId::ExtraHeaderId as _)
            .ok_or(Error::UnknownSection(AssemblySectionId::ExtraHeaderId as _))?;
        let extra_header = ExtraHeader::read_from_section(&mut Cursor::new(extra_header_section))?;
        let custom_attributes =
            file.read_all::<CustomAttribute>(AssemblySectionId::CustomAttributes as _)?;
        let type_refs = file.read_all::<TypeRef>(AssemblySectionId::TypeRefs as _)?;
        let type_specs = file.read_all::<TypeSpec>(AssemblySectionId::TypeSpecs as _)?;
        let method_specs = file.read_all::<MethodSpec>(AssemblySectionId::MethodSpecs as _)?;
        let type_defs = file.read_all::<TypeDef>(AssemblySectionId::TypeDefs as _)?;

        Ok(Self {
            string_section: string_section.clone(),
            extra_header,
            custom_attributes,
            type_refs,
            type_specs,
            method_specs,
            type_defs,
        })
    }
    pub fn from_bytes(bytes: Vec<u8>) -> binary_core::BinaryResult<Self> {
        FileBuilder::from_bytes(bytes).and_then(|file| AssemblyBuilder::from_file(file))
    }
    pub fn from_path<P: AsRef<Path>>(p: P) -> binary_core::BinaryResult<Self> {
        let bytes = std::fs::read(p)?;
        AssemblyBuilder::from_bytes(bytes)
    }

    pub fn get_string(&self, string_ref: StringRef) -> binary_core::BinaryResult<&str> {
        self.string_section
            .as_string_section()
            .get_string(string_ref)
            .ok_or_else(|| Error::UnknownStringRef(string_ref))
    }

    pub fn add_string(&mut self, s: &str) -> StringRef {
        self.string_section.as_string_section_mut().add_string(s)
    }
    pub fn add_type_ref(&mut self, tyr: TypeRef) -> u32 {
        if let Some(pos) = self.type_refs.iter().position(|x| x == &tyr) {
            pos as u32
        } else {
            self.type_refs.push(tyr);
            (self.type_refs.len() - 1) as u32
        }
    }
    pub fn into_file(self) -> binary_core::BinaryResult<FileBuilder> {
        let mut file = FileBuilder::new();
        unsafe {
            file.set_string_section(self.string_section);
        }
        let mut extra_header_section = SectionBuilder::new();
        self.extra_header
            .write_to_section(&mut extra_header_section.construct_mut_vec_cursor())?;
        file.add_section(extra_header_section);

        file.add_section(SectionBuilder::new()); // CustomAttributes
        file.add_section(SectionBuilder::new()); // TypeRefs
        file.add_section(SectionBuilder::new()); // TypeSpecs
        file.add_section(SectionBuilder::new()); // MethodSpecs
        file.add_section(SectionBuilder::new()); // TypeDefs

        file.write_all(
            AssemblySectionId::CustomAttributes as usize,
            &self.custom_attributes,
        )?;
        file.write_all(AssemblySectionId::TypeRefs as _, &self.type_refs)?;
        file.write_all(AssemblySectionId::TypeSpecs as _, &self.type_specs)?;
        file.write_all(AssemblySectionId::MethodSpecs as _, &self.method_specs)?;
        file.write_all(AssemblySectionId::TypeDefs as _, &self.type_defs)?;

        Ok(file)
    }
    pub fn write_to<W: Write + Seek>(self, w: &mut W) -> binary_core::BinaryResult<()> {
        let file = self.into_file()?;
        file.write_to(w).map_err(From::from)
    }
}
