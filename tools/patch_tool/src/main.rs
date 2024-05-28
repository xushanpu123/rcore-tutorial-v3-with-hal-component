mod patch;
#[macro_use]
extern crate anyhow;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!("Usage: cargo run <cargo_path> <patch_name> <patch_repo_name> <commit>");
        return;
    }
    let cargo_path = &args[1];
    let patch_name = &args[2];
    // 默认服务端是 github，后续可能需要根据默认参数来传递
    let patch_repo_name = &args[3];
    let commit = &args[4];
    patch::do_patch(cargo_path, patch_name, patch_repo_name, commit).unwrap();
}
