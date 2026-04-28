mod app;
mod cli;
mod clipboard;
mod config;
mod content;
mod errors;
mod inventory;
mod markdown;
mod output;
mod paths;
#[cfg(test)]
mod test_support;
use std::ffi::OsString;
#[inline]
pub fn run<I>(args: I) -> errors::AppResult<()>
where
    I: IntoIterator<Item = OsString>,
{
    app::run(args)
}
