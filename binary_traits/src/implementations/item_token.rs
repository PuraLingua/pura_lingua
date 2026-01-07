use types::item_token::{ItemToken, MethodToken, TypeToken};

macro impl_($($t:ty)*) {$(
    impl $crate::ReadFromFile for $t {
        fn read_from_file(file: &mut ::types::File) -> global_errors::Result<Self> {
            <u32 as $crate::ReadFromFile>::read_from_file(file)
                .map(|x| unsafe { std::mem::transmute(x) })
        }
    }

    impl $crate::WriteToFile for $t {
        fn write_to_file(&self, file: &mut ::types::File) -> global_errors::Result<()> {
            $crate::WriteToFile::write_to_file(&self.into_bits(), file)
        }
    }
)*}

impl_! {
    ItemToken
    TypeToken
    MethodToken
}
