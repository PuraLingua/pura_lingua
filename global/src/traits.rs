pub const trait IUnwrap<T>: Sized {
    fn _unwrap(self) -> T;
}

const impl<T> IUnwrap<T> for Option<T> {
    fn _unwrap(self) -> T {
        self.unwrap()
    }
}

impl<T, E: std::fmt::Debug> IUnwrap<T> for Result<T, E> {
    fn _unwrap(self) -> T {
        self.unwrap()
    }
}
