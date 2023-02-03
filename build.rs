fn main() {
    println!("cargo:rerun-if-changed=src/windows.c");
    cc::Build::new()
        .file("src/windows.c")
        .warnings_into_errors(true)
        .compile("windowsutil");
}
