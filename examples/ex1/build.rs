fn main() {
    let target = std::env::var("TARGET").unwrap();
    let mut cfg = cmk::Config::new("cpplib");
    cfg.profile("Release");
    if target.contains("emscripten") {
        if let Ok(emsdk) = std::env::var("EMSDK") {
            cfg.define(
                "CMAKE_TOOLCHAIN_FILE",
                std::path::PathBuf::from(emsdk)
                    .join("upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake"),
            );
        }
    }
    cfg.build();
    println!("cargo:rustc-link-lib=cpplib");
}
