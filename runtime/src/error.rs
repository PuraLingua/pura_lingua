use derive_more::Display;

#[derive(Clone, Debug, Display, thiserror::Error)]
pub enum RuntimeError {
    #[display("FailedGetField({_0})")]
    FailedGetField(u32),
    /// 0: Name of the type
    #[display("FailedUnmarshal({_0})")]
    FailedUnmarshal(&'static str),

    NotAnArray,

    #[cfg(windows)]
    #[display("{}", if _0.is_err() {
		format!("WindowsAPIError{:#x}: {}", _0.0, _0.message())
	} else {
		"".to_owned()
	})]
    WindowsAPIError(windows::core::HRESULT),
}

impl RuntimeError {
    pub fn unmarshal_failed<T: ?Sized>() -> Self {
        Self::FailedUnmarshal(std::any::type_name::<T>())
    }
}
