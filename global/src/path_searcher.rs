use std::path::PathBuf;
use std::str::FromStr;

pub fn get_stdlib_dir() -> crate::Result<PathBuf> {
    let home = get_home()?;
    let mut buf = PathBuf::from_str(&home)?;
    buf.push("Library");
    Ok(buf)
}

pub fn get_stdlib_prebuilt_dir() -> crate::Result<PathBuf> {
    let mut buf = get_stdlib_dir()?;
    buf.push("__prebuilt");
    Ok(buf)
}

pub fn get_home() -> crate::Result<String> {
    Ok(std::env::var("PURALINGUA_HOME")?)
}
