fn main() {
    let target = std::env::var("TARGET").unwrap();
    let mut build = cc::Build::new();

    build.file("ffi/ffi.cpp").include("ffi").cpp(true);

    if target.contains("apple-ios") {
        build.flag("-mios-version-min=15.6");
    }

    build.compile("ffi");
}

