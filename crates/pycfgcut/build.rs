#[cfg(windows)]
fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is set");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV is set");

    if target_env != "msvc" {
        return;
    }

    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set"))
        .join("python3-lib");

    // Maturin's abi3 config can point at a Python toolcache directory without python3.lib.
    python3_dll_a::ImportLibraryGenerator::new(&target_arch, &target_env)
        .generate(&out_dir)
        .expect("generate python3 import library");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
}

#[cfg(not(windows))]
fn main() {}
