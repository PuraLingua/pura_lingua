use std::{collections::HashMap, hash::Hash};

use indexmap::{IndexMap, IndexSet};

use crate::traits::{ReadFromSection, WriteToSection};

impl<T: ReadFromSection> ReadFromSection for Vec<T> {
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let len = u64::read_from_section(cursor)?;
        let mut v = Vec::with_capacity(len as usize);
        for _ in 0..len {
            v.push(T::read_from_section(cursor)?);
        }

        Ok(v)
    }
}

impl<T: WriteToSection> WriteToSection for Vec<T> {
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        (self.len() as u64).write_to_section(cursor)?;
        for v in self {
            v.write_to_section(cursor)?;
        }

        Ok(())
    }
}

impl<K, V> ReadFromSection for IndexMap<K, V>
where
    K: ReadFromSection + Hash + Eq,
    V: ReadFromSection,
{
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let len = u64::read_from_section(cursor)?;
        let mut map = Self::with_capacity(len as usize);
        for _ in 0..len {
            map.insert(K::read_from_section(cursor)?, V::read_from_section(cursor)?);
        }

        Ok(map)
    }
}

impl<K, V> WriteToSection for IndexMap<K, V>
where
    K: WriteToSection,
    V: WriteToSection,
{
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        (self.len() as u64).write_to_section(cursor)?;
        for (k, v) in self {
            k.write_to_section(cursor)?;
            v.write_to_section(cursor)?;
        }

        Ok(())
    }
}

impl<K, V> ReadFromSection for HashMap<K, V>
where
    K: ReadFromSection + Hash + Eq,
    V: ReadFromSection,
{
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> Result<Self, crate::error::Error> {
        let len = u64::read_from_section(cursor)?;
        let mut map = Self::with_capacity(len as usize);
        for _ in 0..len {
            map.insert(K::read_from_section(cursor)?, V::read_from_section(cursor)?);
        }

        Ok(map)
    }
}

impl<K, V> WriteToSection for HashMap<K, V>
where
    K: WriteToSection,
    V: WriteToSection,
{
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> Result<(), crate::error::Error> {
        (self.len() as u64).write_to_section(cursor)?;
        for (k, v) in self {
            k.write_to_section(cursor)?;
            v.write_to_section(cursor)?;
        }

        Ok(())
    }
}

impl<T> ReadFromSection for IndexSet<T>
where
    T: ReadFromSection + Hash + Eq,
{
    fn read_from_section(
        cursor: &mut std::io::Cursor<&crate::section::Section>,
    ) -> crate::BinaryResult<Self> {
        let len = u64::read_from_section(cursor)?;
        let mut set = Self::with_capacity(len as usize);
        for _ in 0..len {
            set.insert(T::read_from_section(cursor)?);
        }
        Ok(set)
    }
}

impl<T> WriteToSection for IndexSet<T>
where
    T: WriteToSection,
{
    fn write_to_section(
        &self,
        cursor: &mut std::io::Cursor<&mut Vec<u8>>,
    ) -> crate::BinaryResult<()> {
        (self.len() as u64).write_to_section(cursor)?;
        for v in self {
            v.write_to_section(cursor)?;
        }
        Ok(())
    }
}
