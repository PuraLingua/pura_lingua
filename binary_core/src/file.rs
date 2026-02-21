use std::io::{Cursor, Seek, Write};

use crate::{
    error::Error,
    section::{Section, SectionInfo},
    traits::{ReadFromSection, StringRef, WriteToSection},
};

pub struct FileParser<'a> {
    bytes: &'a [u8],
    index: usize,
}

#[repr(C, packed)]
pub struct Header {
    magic: [u8; 2],
    version: [u8; 2],
    section_info_len: u32,
    section_infos: [SectionInfo],
}

impl Header {
    const SECTION_INFOS_OFFSET: usize =
        std::mem::offset_of!(Self, section_info_len) + size_of::<u32>();

    const SIZE_WITHOUT_SECTION_INFO: usize = Self::SECTION_INFOS_OFFSET;
}

const CURRENT_MAGIC: [u8; 2] = *b"PL";
const CURRENT_VERSION: [u8; 2] = [0x00, 0x00];

impl<'a> FileParser<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.bytes.as_ptr()
    }

    pub fn parse_header(&mut self) -> Result<&'a Header, Error> {
        debug_assert!(self.index == 0);
        if self.bytes.len() < Header::SECTION_INFOS_OFFSET {
            #[cfg(debug_assertions)]
            {
                println!("Header not satisfied");
            }
            return Err(Error::WrongFileSize);
        }

        let section_info_len = u32::from_le_bytes(
            *self.bytes[std::mem::offset_of!(Header, section_info_len)
                ..(std::mem::offset_of!(Header, section_info_len) + size_of::<u32>())]
                .as_array::<{ size_of::<u32>() }>()
                .unwrap(),
        );
        let size = Header::SIZE_WITHOUT_SECTION_INFO + (section_info_len as usize);
        if self.bytes.len() < size {
            #[cfg(debug_assertions)]
            {
                println!("SectionInfo not satisfied");
            }
            return Err(Error::WrongFileSize);
        }
        self.index = size;
        unsafe {
            Ok(&*std::ptr::from_raw_parts(
                self.as_ptr(),
                section_info_len as usize,
            ))
        }
    }
}

#[derive(Debug)]
pub struct File {
    version: [u8; 2],
    sections: Vec<Section>,
}

impl File {
    pub const STRING_SECTION: usize = 0;
    pub const FIRST_NON_STANDARD_AVAILABLE_SECTION_ID: usize = 1;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            version: CURRENT_VERSION,
            sections: vec![Section::new()],
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let mut parser = FileParser::from_bytes(&bytes);
        let header = parser.parse_header()?;

        let mut sections = Vec::with_capacity(header.section_info_len as usize);
        for section_info in &header.section_infos {
            sections.push(Section::with_bytes(
                parser.bytes[(section_info.offset as usize)
                    ..((section_info.offset + section_info.len) as usize)]
                    .to_vec(),
            ));
        }

        Ok(Self {
            version: header.version,
            sections,
        })
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    pub fn get_section(&self, index: usize) -> Option<&Section> {
        self.sections.get(index)
    }

    /// # Safety
    /// After this, every [`StringRef`] created by [`Self::add_string`] will be invalid,
    /// although [`StringRef`] itself does not know.
    pub unsafe fn set_string_section(&mut self, section: Section) {
        self.sections[Self::STRING_SECTION] = section;
    }

    pub fn add_string(&mut self, s: &str) -> StringRef {
        self.sections
            .get_mut(Self::STRING_SECTION)
            .unwrap()
            .as_string_section_mut()
            .add_string(s)
    }

    pub fn get_string(&self, string_ref: StringRef) -> Option<&str> {
        self.sections
            .get(Self::STRING_SECTION)?
            .as_string_section()
            .get_string(string_ref)
    }

    pub fn write_to<W: Write + Seek>(self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&CURRENT_MAGIC)?;
        w.write_all(&self.version)?;
        w.write_all(&(self.sections.len() as u32).to_le_bytes())?;
        let mut current_section_info_offset = w.stream_position()?;
        w.write_all(&vec![0; self.sections.len() * size_of::<SectionInfo>()])?;
        for section in self.sections {
            let offset = w.stream_position()?;
            let len = section.len();
            w.seek(std::io::SeekFrom::Start(current_section_info_offset))?;
            w.write_all(&offset.to_le_bytes())?;
            w.write_all(&len.to_le_bytes())?;
            current_section_info_offset = w.stream_position()?;
            w.seek(std::io::SeekFrom::Start(offset))?;
            w.write_all(section.as_bytes())?;
        }

        Ok(())
    }

    pub fn read_all<T: ReadFromSection>(
        &self,
        section_id: usize,
    ) -> Result<Vec<T>, crate::error::Error> {
        let section = self
            .sections
            .get(section_id)
            .ok_or(Error::UnknownSection(section_id))?;
        let mut cursor = Cursor::new(section);
        let mut result = Vec::new();

        loop {
            if cursor.position() >= section.len() {
                break Ok(result);
            }
            result.push(T::read_from_section(&mut cursor)?);
        }
    }
    pub fn write_all<T: WriteToSection>(
        &mut self,
        section_id: usize,
        values: &[T],
    ) -> Result<(), Error> {
        let section = self
            .sections
            .get_mut(section_id)
            .ok_or(Error::WrongFileSize)?;
        let mut cursor = section.construct_mut_vec_cursor();
        for value in values {
            value.write_to_section(&mut cursor)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use super::*;

    #[test]
    fn test_header() -> Result<(), Error> {
        let mut file = File::new();
        file.add_section(Section::with_bytes(vec![0, 1, 2, 3, 4]));
        file.add_section(Section::with_bytes(0xffffu32.to_le_bytes().to_vec()));
        let mut bytes = Cursor::new(Vec::<u8>::new());
        file.write_to(&mut bytes)?;
        let file_gotten = File::from_bytes(bytes.into_inner())?;
        #[derive(Debug)]
        #[allow(unused)]
        struct TestData(u32);
        impl ReadFromSection for TestData {
            fn read_from_section(
                cursor: &mut Cursor<&Section>,
            ) -> Result<Self, crate::error::Error> {
                let mut this = [0u8; size_of::<u32>()];
                cursor.read_exact(&mut this)?;
                Ok(Self(u32::from_le_bytes(this)))
            }
        }
        dbg!(&file_gotten);
        dbg!(file_gotten.read_all::<TestData>(2)?);

        Ok(())
    }

    #[test]
    fn test_string() -> Result<(), Error> {
        let mut file = File::new();
        let aaa = file.add_string("aaa");
        let bbb = file.add_string("bbb");
        let ccc = file.add_string("ccc");
        let ddd = file.add_string("ddd");
        let mut bytes = Cursor::new(Vec::<u8>::new());
        file.write_to(&mut bytes)?;
        let file_gotten = File::from_bytes(bytes.into_inner())?;
        dbg!(file_gotten.get_string(aaa));
        dbg!(file_gotten.get_string(bbb));
        dbg!(file_gotten.get_string(ccc));
        dbg!(file_gotten.get_string(ddd));

        Ok(())
    }
}
