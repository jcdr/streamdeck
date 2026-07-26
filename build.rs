fn main() {
    let manifest_directory = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let local_library_directory = format!("{manifest_directory}/.deps/lib");
    println!("cargo:rustc-link-search=native={local_library_directory}");
    println!("cargo:rerun-if-changed=.deps/lib");
}
