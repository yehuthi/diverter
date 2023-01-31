fn main() {
    println!("cargo:rerun-if-changed=src/windows.c");
    cc::Build::new()
        .file("src/windows.c")
        .compile("windowsutil");
}
