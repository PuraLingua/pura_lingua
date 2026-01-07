use std::path::Path;

use global::getset::{Getters, MutGetters};
use global::{IndexMap, StringName};
use proc_macros::{ReadFromFile, WriteToFile};

use crate::core::{File, FileExt};
use crate::custom_attribute::CustomAttribute;
use crate::implement::Implementation;
use crate::method::MethodSpec;
use crate::ty::TypeDef;
use crate::{Error, TypeRef, TypeSpec};
use traits::{ReadFromFile, WriteToFile};

#[derive(Default, Debug, Clone, ReadFromFile, WriteToFile, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Assembly {
    name: StringName,
    custom_attributes: Vec<CustomAttribute>,
    type_defs: IndexMap<StringName, TypeDef>,
    type_refs: Vec<TypeRef>,
    type_specs: Vec<TypeSpec>,
    method_specs: Vec<MethodSpec>,
    implementations: IndexMap<StringName, Implementation>,
}

const MAGIC: [u8; 2] = *b"PL";

#[allow(unused)]
#[derive(Debug, Default, Clone, ReadFromFile, WriteToFile)]
struct Header {
    magic: [u8; 2],
}

#[allow(unused)]
impl Header {
    fn check(&self) -> global::Result<()> {
        if self.magic != MAGIC {
            return Err(Error::WrongFileFormat.into());
        }
        Ok(())
    }
}

impl Assembly {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> global::Result<Self> {
        let mut file = File::new(bytes)?;
        Self::read_from_file(&mut file)
    }
    pub fn from_file<P: AsRef<Path>>(p: P) -> global::Result<Self> {
        Self::from_bytes(std::fs::read(p)?)
    }

    pub fn to_file_bytes(&self) -> global::Result<Vec<u8>> {
        let mut file = File::default();
        self.write_to_file(&mut file)?;
        file.to_bytes()
    }
}
