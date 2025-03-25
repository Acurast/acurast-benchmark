fn main() {
    cc::Build::new()
        .file("ffi/ffi.cpp")
        .include("ffi")
        .cpp(true)
        .compile("ffi");
}