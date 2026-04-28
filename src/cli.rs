use std::{env, ffi::OsString, io, path::PathBuf};
pub(crate) fn input_path_from_args<I>(raw_args: I) -> io::Result<PathBuf>
where
    I: IntoIterator<Item = OsString>,
{
    let mut arguments = raw_args.into_iter();
    let _program = arguments.next();
    let input_path = arguments.next();
    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "只接受一个项目路径参数",
        ));
    }
    input_path.map_or_else(env::current_dir, |path| Ok(PathBuf::from(path)))
}
#[cfg(test)]
mod tests;
