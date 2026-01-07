use global::IndexSet;
use std::io::{Cursor, Read};

pub use traits::{ReadFromFile, WriteToFile};
pub use types::{File, StringInterner, integer::CompressedU32};

pub trait FileExt: Sized {
    fn new<T: AsRef<[u8]>>(bytes: T) -> global::Result<Self>;
    fn to_bytes(&self) -> global::Result<Vec<u8>>;
}

impl FileExt for types::File {
    fn new<T: AsRef<[u8]>>(bytes: T) -> global::Result<Self> {
        let mut data = Cursor::new(bytes.as_ref().to_vec());
        let mut interner_len = [0u8; 8];
        data.read_exact(&mut interner_len)?;
        let interner_len = u64::from_le_bytes(interner_len);
        let mut interner = vec![0u8; interner_len as usize];
        data.read_exact(&mut interner)?;
        let interner = StringInterner::new(interner);
        let mut _data = Vec::new();
        data.read_to_end(&mut _data)?;
        Ok(Self {
            interner,
            data: Cursor::new(_data),
        })
    }

    fn to_bytes(&self) -> global::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let interner_bytes = self.interner.to_bytes();
        buf.extend_from_slice(&(interner_bytes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.interner.to_bytes());
        buf.extend_from_slice(self.data.get_ref());
        Ok(buf)
    }
}

pub trait StringInternerExt: Sized {
    fn new<T: AsRef<[u8]>>(bytes: T) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

impl StringInternerExt for types::StringInterner {
    fn new<T: AsRef<[u8]>>(bytes: T) -> Self {
        let bytes = bytes.as_ref();
        let mut set = IndexSet::new();
        if !bytes.starts_with(b"\0") {
            set.insert("".to_owned());
        }
        for s in bytes.split(|x| x.eq(&b'\0')) {
            let s = unsafe { String::from_utf8_unchecked(Vec::from(s)) };
            set.insert(s);
        }
        Self { set }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut s = self.set.iter().fold(Vec::<u8>::new(), |mut a, b| {
            let mut s = b.as_bytes().to_vec();
            s.push(b'\0');
            a.extend_from_slice(&s);
            a
        });
        s.pop();
        s.extend_from_slice(&vec![0; s.len() % 8]);
        s
    }
}

#[cfg(test)]
mod test_compressed {
    use types::integer::CompressedU32;

    use super::*;

    #[test]
    fn t_u32() -> Result<(), global::Error> {
        let x = (0u32..536870912).collect::<Vec<_>>();
        let mut file = File::new(vec![0x00; size_of::<u64>()])?;
        for i in &x {
            match CompressedU32(*i).write_to_file(&mut file) {
                Ok(_) => {}
                Err(_) => {
                    println!("Cannot write {i}")
                }
            }
        }

        file.data.set_position(0);

        for i in &x {
            match CompressedU32::read_from_file(&mut file) {
                Ok(x) if x.0.eq(i) => {}
                Ok(x) => {
                    println!("Unmatched got: {} expect: {i}", x.0)
                }
                Err(_) => {
                    println!("Cannot read {i}")
                }
            }
        }

        Ok(())
    }
}
