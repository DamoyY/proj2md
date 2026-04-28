use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
fn main() {
    if let Err(err) = proj2md::run(std::env::args_os()) {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}
