// Rebuild when embedded package files change (include_dir does not track them).
fn main() {
    for p in ["../templates", "../plugin", "../tools.toml", "../mise.toml"] {
        println!("cargo:rerun-if-changed={p}");
    }
}
