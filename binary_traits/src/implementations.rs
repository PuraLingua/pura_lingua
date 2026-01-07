use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::sync::Arc;

use const_for::const_for;
use enumflags2::{BitFlag, BitFlags};
use indexmap::IndexMap;
use string_name::StringName;

use crate::{ReadFromFile, WriteToFile};
use types::File;

mod item_token;
mod pointers;
mod primitive;

impl ReadFromFile for String {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let i = u64::read_from_file(file)?;
        Ok(file.get_string(i)?.to_owned())
    }
}

impl WriteToFile for str {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        let i = file.string_position_of(self)?;
        i.write_to_file(file)
    }
}

impl ReadFromFile for StringName {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        Ok(Self::from_string(String::read_from_file(file)?))
    }
}
impl WriteToFile for StringName {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        self.as_str().write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Vec<T> {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut vec = Vec::new();
        for _ in 0..i {
            vec.push(T::read_from_file(file)?);
        }
        Ok(vec)
    }
}

impl<T: WriteToFile> WriteToFile for Vec<T> {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (**self).write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Arc<[T]> {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        Vec::read_from_file(file).map(Self::from)
    }
}

impl<T: WriteToFile> WriteToFile for [T] {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter().try_for_each(|item| item.write_to_file(file))
    }
}

impl<T: ReadFromFile, const N: usize> ReadFromFile for [T; N] {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let mut this = std::array::from_fn(|_| MaybeUninit::<T>::uninit());
        const_for! {
            i in (0..N) => {
                this[i] = MaybeUninit::new(T::read_from_file(file)?);
            }
        }
        unsafe { Ok(MaybeUninit::array_assume_init(this)) }
    }
}

impl<T: WriteToFile, const N: usize> WriteToFile for [T; N] {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        const_for! {
            i in (0..N) => {
                self[i].write_to_file(file)?;
            }
        }
        Ok(())
    }
}

impl<K: ReadFromFile + Eq + Hash, V: ReadFromFile> ReadFromFile for HashMap<K, V> {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut map = Self::with_capacity(i as usize);
        for _ in 0..i {
            let k = K::read_from_file(file)?;
            let v = V::read_from_file(file)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

impl<K: WriteToFile, V: WriteToFile> WriteToFile for HashMap<K, V> {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter()
            .try_for_each(|(k, v)| k.write_to_file(file).and_then(|_| v.write_to_file(file)))
    }
}

impl<K: ReadFromFile + Eq + Hash, V: ReadFromFile> ReadFromFile for IndexMap<K, V> {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut map = Self::with_capacity(i as usize);
        for _ in 0..i {
            let k = K::read_from_file(file)?;
            let v = V::read_from_file(file)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

impl<K: WriteToFile + Any, V: WriteToFile + Any> WriteToFile for IndexMap<K, V> {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter()
            .try_for_each(|(k, v)| k.write_to_file(file).and_then(|_| v.write_to_file(file)))
    }
}

impl<T: BitFlag> ReadFromFile for BitFlags<T>
where
    T::Numeric: ReadFromFile + Send + Sync + Debug,
    T: Send + Sync + Debug,
{
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let num = T::Numeric::read_from_file(file)?;
        Ok(Self::from_bits(num)?)
    }
}

impl<T: BitFlag> WriteToFile for BitFlags<T>
where
    T::Numeric: WriteToFile,
{
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        let num = self.bits();
        num.write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Option<T> {
    fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
        let b = u8::read_from_file(file)?;
        if b == 1 {
            Ok(Some(T::read_from_file(file)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: WriteToFile> WriteToFile for Option<T> {
    fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
        match self {
            Some(val) => 1u8
                .write_to_file(file)
                .and_then(|_| val.write_to_file(file)),
            None => 0u8.write_to_file(file),
        }
    }
}

macro_rules! impl_for_tuple {
    () => {
        impl WriteToFile for () {
            fn write_to_file(&self, _: &mut File) -> global_errors::Result<()> {
                Ok(())
            }
        }
        impl ReadFromFile for () {
            fn read_from_file(_: &mut File) -> global_errors::Result<Self> {
                Ok(())
            }
        }
    };
    ($first:ident $( $i:ident)*) => {
        impl_for_tuple!($($i) *);

        #[allow(non_snake_case)]
        impl<$first: WriteToFile $(, $i: WriteToFile)*> WriteToFile for ($first, $($i),*) {
            fn write_to_file(&self, file: &mut File) -> global_errors::Result<()> {
                let ($first, $($i),*) = self;
                $first.write_to_file(file)?;
                $( $i.write_to_file(file)?; )*
                Ok(())
            }
        }
        #[allow(non_snake_case)]
        impl<$first: ReadFromFile $(, $i: ReadFromFile)*> ReadFromFile for ($first, $($i),*) {
            fn read_from_file(file: &mut File) -> global_errors::Result<Self> {
                let $first = $first::read_from_file(file)?;
                $( let $i = $i::read_from_file(file)?; )*
                Ok(($first, $($i),*))
            }
        }
    };
}

impl_for_tuple!(
    A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
);
