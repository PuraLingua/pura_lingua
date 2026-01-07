use std::sync::Arc;

use crate::{ReadFromFile, WriteToFile};

impl<T: ReadFromFile> ReadFromFile for Box<T> {
    fn read_from_file(file: &mut types::File) -> global_errors::Result<Self> {
        T::read_from_file(file).map(Self::new)
    }
}

impl<T: WriteToFile> WriteToFile for Box<T> {
    fn write_to_file(&self, file: &mut types::File) -> global_errors::Result<()> {
        (**self).write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Arc<T> {
    fn read_from_file(file: &mut types::File) -> global_errors::Result<Self> {
        T::read_from_file(file).map(Self::new)
    }
}

impl<T: WriteToFile> WriteToFile for Arc<T> {
    fn write_to_file(&self, file: &mut types::File) -> global_errors::Result<()> {
        (**self).write_to_file(file)
    }
}
