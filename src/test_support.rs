use core::{
    fmt::{Debug, Display},
    sync::atomic::{AtomicUsize, Ordering},
};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
pub(crate) struct TestDir {
    path: PathBuf,
}
impl TestDir {
    pub(crate) fn new(name: &str) -> io::Result<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!("proj2md-test-{name}-{}-{id}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn write_bytes(&self, relative_path: &str, bytes: &[u8]) -> io::Result<PathBuf> {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)?;
        Ok(path)
    }
    pub(crate) fn write_str(&self, relative_path: &str, content: &str) -> io::Result<PathBuf> {
        self.write_bytes(relative_path, content.as_bytes())
    }
    pub(crate) fn create_dir(&self, relative_path: &str) -> io::Result<PathBuf> {
        let path = self.path.join(relative_path);
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        if self.path.exists()
            && let Err(err) = fs::remove_dir_all(&self.path)
        {
            panic!("删除测试目录失败: {}: {err}", self.path.display());
        }
    }
}
pub(crate) fn must<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: Display,
{
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: {err}"),
    }
}
pub(crate) fn must_err<T, E>(result: Result<T, E>, context: &str) -> E
where
    T: Debug,
{
    match result {
        Ok(value) => panic!("{context}: {value:?}"),
        Err(err) => err,
    }
}
