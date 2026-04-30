use std::path::{Path, PathBuf};

pub struct ScopeCdGuard {
    original: PathBuf,
}

impl Drop for ScopeCdGuard {
    fn drop(&mut self) {
        #[expect(unused_must_use)]
        std::env::set_current_dir(&self.original);
    }
}

#[must_use = "Or it does nothing"]
pub fn scope_cd(path: impl AsRef<Path>) -> std::io::Result<ScopeCdGuard> {
    let original = std::env::current_dir()?;

    std::env::set_current_dir(path)?;

    Ok(ScopeCdGuard { original })
}
