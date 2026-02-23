use std::{
    io::{Cursor, Seek, Write},
    path::Path,
};

use binary_core::{
    Error,
    file::File,
    section::Section,
    traits::{ReadFromSection as _, StringRef, WriteToSection},
};
use derive_more::Debug;
use proc_macros::{ReadFromSection, WriteToSection};

use crate::{
    custom_attribute::CustomAttribute,
    ty::{MethodSpec, TypeDef, TypeRef, TypeSpec},
};

#[derive(Debug)]
pub struct Assembly {
    #[debug(
        "ExtraHeader {{ name: {} }}",
        string_section
            .as_string_section()
            .get_string(extra_header.name)
            .unwrap()
    )]
    pub extra_header: ExtraHeader,
    #[debug(skip)]
    pub string_section: Section,
    pub custom_attributes: Vec<CustomAttribute>,
    pub type_refs: Vec<TypeRef>,
    pub type_specs: Vec<TypeSpec>,
    pub method_specs: Vec<MethodSpec>,
    pub type_defs: Vec<TypeDef>,
}

#[derive(ReadFromSection, WriteToSection, Debug, Clone)]
pub struct ExtraHeader {
    pub name: StringRef,
}

#[repr(usize)]
enum AssemblySectionId {
    ExtraHeaderId = binary_core::file::File::FIRST_NON_STANDARD_AVAILABLE_SECTION_ID,
    CustomAttributes,
    TypeRefs,
    TypeSpecs,
    MethodSpecs,
    TypeDefs,
}

impl Assembly {
    pub fn new(name: &str) -> Self {
        let mut string_section = Section::new();
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
    pub fn from_file(file: &File) -> binary_core::BinaryResult<Self> {
        let string_section = file.get_section(File::STRING_SECTION).unwrap();
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
        File::from_bytes(bytes).and_then(|file| Assembly::from_file(&file))
    }
    pub fn from_path<P: AsRef<Path>>(p: P) -> binary_core::BinaryResult<Self> {
        let bytes = std::fs::read(p)?;
        Self::from_bytes(bytes)
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
    pub fn into_file(self) -> binary_core::BinaryResult<File> {
        let mut file = File::new();
        unsafe {
            file.set_string_section(self.string_section);
        }
        let mut extra_header_section = Section::new();
        self.extra_header
            .write_to_section(&mut extra_header_section.construct_mut_vec_cursor())?;
        file.add_section(extra_header_section);

        file.add_section(Section::new()); // CustomAttributes
        file.add_section(Section::new()); // TypeRefs
        file.add_section(Section::new()); // TypeSpecs
        file.add_section(Section::new()); // MethodSpecs
        file.add_section(Section::new()); // TypeDefs

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
