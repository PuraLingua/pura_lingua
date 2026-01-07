use std::path::PathBuf;
use std::str::FromStr;

pub fn get_stdlib_dir() -> crate::Result<String> {
    let home = get_home()?;
    let mut buf = PathBuf::from_str(&home)?;
    buf.push("Library");
    Ok(buf.to_str().unwrap().to_owned())
}

pub fn get_home() -> crate::Result<String> {
    Ok(std::env::var("PURALINGUA_HOME")?)
}
