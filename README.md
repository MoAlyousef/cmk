# cmk

A simpler implementation of cmake-rs which assumes a recent enough version of CMake.

## Usage

```toml
# Cargo.toml
[build-dependencies]
cmk = "0.1"
```

## Example
```rust,no_run
// build.rs
fn main() {
    let dst = cmk::Config::new("cpplib")
        .generator("Ninja")
        .profile("Release")
        .define("SOME_CMAKE_OPTION", "ON")
        .build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=cpplib");
}
```