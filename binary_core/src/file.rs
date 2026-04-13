use std::io::{Cursor, Seek, Write};

use crate::{
    error::Error,
    section::{Section, SectionBuilder, SectionInfo},
    traits::{ReadFromSection, StringRef, WriteToSection},
};

#[derive(Debug)]
pub struct FileParser<'a> {
    bytes: &'a [u8],
    content_index: usize,
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
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        let mut this = Self {
            bytes,
            content_index: 0,
        };
        this.render_header()?;
        Ok(this)
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.bytes.as_ptr()
    }

    fn render_header(&mut self) -> Result<(), Error> {
        debug_assert!(self.content_index == 0);
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
        self.content_index = size;
        Ok(())
    }

    pub const fn get_header(&self) -> &'a Header {
        unsafe { &*std::ptr::from_raw_parts(self.as_ptr(), self.content_index) }
    }

    pub fn section_iter(self) -> Result<SectionIter<'a>, Error> {
        SectionIter::new(self)
    }
}

pub struct SectionIter<'a> {
    header: &'a Header,
    raw: &'a [u8],
    section_index: usize,
}

impl<'a> SectionIter<'a> {
    fn new(parser: FileParser<'a>) -> Result<Self, Error> {
        Ok(Self {
            header: parser.get_header(),
            raw: parser.bytes,
            section_index: 0,
        })
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = &'a Section;

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.header.section_infos.get(self.section_index)?;
        let res = Section::with_bytes(
            &self.raw[(info.offset as usize)..((info.offset + info.len) as usize)],
        );
        self.section_index += 1;
        Some(res)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.header.section_infos.len(),
            Some(self.header.section_infos.len()),
        )
    }
}

unsafe impl<'a> std::iter::TrustedLen for SectionIter<'a> {}

#[derive(Debug)]
pub struct File<'a> {
    raw: FileParser<'a>,
}

#[repr(usize)]
pub enum PredefinedSectionId {
    String,

    FirstNonStandard,
}

impl<'a> File<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(Self {
            raw: FileParser::from_bytes(bytes)?,
        })
    }

    pub const fn get_section(&self, index: usize) -> Option<&'a Section> {
        let info = self.raw.get_header().section_infos.get(index)?;
        Some(Section::with_bytes(
            &self.raw.bytes[(info.offset as usize)..((info.offset + info.len) as usize)],
        ))
    }
    pub const fn get_predefined_section(&self, id: PredefinedSectionId) -> Option<&'a Section> {
        self.get_section(id as usize)
    }

    pub const fn get_string(&self, string_ref: StringRef) -> Option<&str> {
        self.get_predefined_section(PredefinedSectionId::String)?
            .as_string_section()
            .get_string(string_ref)
    }

    pub fn read_all<T: ReadFromSection>(
        &self,
        section_id: usize,
    ) -> Result<Vec<T>, crate::error::Error> {
        let section = self
            .get_section(section_id)
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
}

#[derive(Debug)]
pub struct FileBuilder {
    pub version: [u8; 2],
    sections: Vec<SectionBuilder>,
}

impl FileBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            version: CURRENT_VERSION,
            sections: vec![SectionBuilder::new()],
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let section_iter = FileParser::from_bytes(&bytes)?.section_iter()?;

        Ok(Self {
            version: section_iter.header.version,
            sections: section_iter.map(ToOwned::to_owned).collect(),
        })
    }

    pub fn get_section(&self, index: usize) -> Option<&SectionBuilder> {
        self.sections.get(index)
    }
    pub fn get_predefined_section(&self, id: PredefinedSectionId) -> Option<&SectionBuilder> {
        self.get_section(id as usize)
    }

    pub fn get_string(&self, string_ref: StringRef) -> Option<&str> {
        self.sections
            .get(PredefinedSectionId::String as usize)?
            .as_string_section()
            .get_string(string_ref)
    }

    pub fn read_all<T: ReadFromSection>(
        &self,
        section_id: usize,
    ) -> Result<Vec<T>, crate::error::Error> {
        let section = self
            .sections
            .get(section_id)
            .ok_or(Error::UnknownSection(section_id))?;
        let mut cursor = Cursor::new(&**section);
        let mut result = Vec::new();

        loop {
            if cursor.position() >= section.len() {
                break Ok(result);
            }
            result.push(T::read_from_section(&mut cursor)?);
        }
    }

    /// # Safety
    /// After this, every [`StringRef`] created by [`Self::add_string`] will be invalid,
    /// although [`StringRef`] itself does not know.
    pub unsafe fn set_string_section(&mut self, section: SectionBuilder) {
        self.sections[PredefinedSectionId::String as usize] = section;
    }

    pub fn add_string(&mut self, s: &str) -> StringRef {
        self.sections
            .get_mut(PredefinedSectionId::String as usize)
            .unwrap()
            .as_string_section_mut()
            .add_string(s)
    }

    pub fn add_section(&mut self, section: SectionBuilder) {
        self.sections.push(section);
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
        let mut file_builder = FileBuilder::new();
        file_builder.add_section(SectionBuilder::with_bytes(vec![0, 1, 2, 3, 4]));
        file_builder.add_section(SectionBuilder::with_bytes(0xffffu32.to_le_bytes().to_vec()));
        let mut bytes = Cursor::new(Vec::<u8>::new());
        file_builder.write_to(&mut bytes)?;
        let file_gotten = File::from_bytes(bytes.get_ref())?;
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
        let mut file = FileBuilder::new();
        let aaa = file.add_string("aaa");
        let bbb = file.add_string("bbb");
        let ccc = file.add_string("ccc");
        let ddd = file.add_string("ddd");
        let mut bytes = Cursor::new(Vec::<u8>::new());
        file.write_to(&mut bytes)?;
        let file_gotten = File::from_bytes(bytes.get_ref())?;
        dbg!(file_gotten.get_string(aaa));
        dbg!(file_gotten.get_string(bbb));
        dbg!(file_gotten.get_string(ccc));
        dbg!(file_gotten.get_string(ddd));

        Ok(())
    }
}
