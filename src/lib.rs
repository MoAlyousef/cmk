#![doc = include_str!("../README.md")]
#![allow(clippy::needless_doctest_main)]

use std::env;
use std::ffi::OsStr;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default, Clone)]
pub struct Config {
    path: PathBuf,
    profile: String,
    args: Vec<String>,
    cflags: String,
    cxxflags: String,
    uses_toolchain_file: bool,
    defined_system_name: bool,
}

impl Config {
    pub fn new<P: AsRef<Path>>(p: P) -> Config {
        Config {
            path: PathBuf::from(p.as_ref()),
            ..Default::default()
        }
    }
    pub fn generator<S: AsRef<str>>(&mut self, generator: S) -> &mut Config {
        self.args.push(format!("-G{}", generator.as_ref()));
        self
    }
    pub fn profile(&mut self, prof: &str) -> &mut Config {
        self.profile = prof.to_string();
        self
    }
    pub fn define<K: AsRef<OsStr>, V: AsRef<OsStr>>(&mut self, key: K, val: V) -> &mut Config {
        if key.as_ref() == "CMAKE_TOOLCHAIN_FILE" {
            self.uses_toolchain_file = true;
        }
        if key.as_ref() == "CMAKE_SYSTEM_NAME" {
            self.defined_system_name = true;
        }
        self.args.push(format!(
            "-D{}={}",
            key.as_ref().to_str().unwrap(),
            val.as_ref().to_str().unwrap()
        ));
        self
    }
    pub fn cflag(&mut self, flag: &str) -> &mut Config {
        self.cflags.push_str(flag);
        self.cflags.push(' ');
        self
    }
    pub fn cxxflag(&mut self, flag: &str) -> &mut Config {
        self.cxxflags.push_str(flag);
        self.cxxflags.push(' ');
        self
    }
    fn uses_toolchain_file(&self) -> bool {
        self.uses_toolchain_file || env::var("CMAKE_TOOLCHAIN_FILE").is_ok()
    }
    fn defined_system_name(&self) -> bool {
        self.defined_system_name || env::var("CMAKE_SYSTEM_NAME").is_ok()
    }
    pub fn try_build(&mut self) -> Result<PathBuf, Error> {
        let target = env::var("TARGET").unwrap();
        let host = env::var("HOST").unwrap();
        let dir_stem = PathBuf::from(self.path.file_stem().unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not defined!"));
        let bin_dir = out_dir.join(dir_stem.join("bin"));
        let lib_dir = out_dir.join(dir_stem.join("lib"));
        if target != host && !self.uses_toolchain_file() && !self.defined_system_name() {
            // copied from https://github.com/rust-lang/cmake-rs/blob/master/src/lib.rs#L450
            let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
            let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
            let (system_name, system_processor) = match (os.as_str(), arch.as_str()) {
                ("android", "arm") => ("Android", "armv7-a"),
                ("android", "x86") => ("Android", "i686"),
                ("android", arch) => ("Android", arch),
                ("dragonfly", arch) => ("DragonFly", arch),
                ("macos", "aarch64") => ("Darwin", "arm64"),
                ("macos", arch) => ("Darwin", arch),
                ("freebsd", "x86_64") => ("FreeBSD", "amd64"),
                ("freebsd", arch) => ("FreeBSD", arch),
                ("fuchsia", arch) => ("Fuchsia", arch),
                ("haiku", arch) => ("Haiku", arch),
                ("ios", "aarch64") => ("iOS", "arm64"),
                ("ios", arch) => ("iOS", arch),
                ("linux", arch) => {
                    let name = "Linux";
                    match arch {
                        "powerpc" => (name, "ppc"),
                        "powerpc64" => (name, "ppc64"),
                        "powerpc64le" => (name, "ppc64le"),
                        _ => (name, arch),
                    }
                }
                ("netbsd", arch) => ("NetBSD", arch),
                ("openbsd", "x86_64") => ("OpenBSD", "amd64"),
                ("openbsd", arch) => ("OpenBSD", arch),
                ("solaris", arch) => ("SunOS", arch),
                ("tvos", "aarch64") => ("tvOS", "arm64"),
                ("tvos", arch) => ("tvOS", arch),
                ("visionos", "aarch64") => ("visionOS", "arm64"),
                ("visionos", arch) => ("visionOS", arch),
                ("watchos", "aarch64") => ("watchOS", "arm64"),
                ("watchos", arch) => ("watchOS", arch),
                ("windows", "x86_64") => ("Windows", "AMD64"),
                ("windows", "x86") => ("Windows", "X86"),
                ("windows", "aarch64") => ("Windows", "ARM64"),
                ("emscripten", "wasm32") => ("Emscripten", "x86"),
                ("none", arch) => ("Generic", arch),
                // Others
                (os, arch) => (os, arch),
            };
            self.define("CMAKE_SYSTEM_NAME", system_name);
            self.define("CMAKE_SYSTEM_PROCESSOR", system_processor);
        }

        // configure
        let mut cmd = Command::new("cmake");
        self.args.push(format!("-S{}", self.path.display()));
        self.args.push(format!("-B{}", bin_dir.display()));
        self.args.push(format!(
            "-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={}",
            lib_dir.display()
        ));
        self.args.push(format!(
            "-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={}",
            lib_dir.display()
        ));
        self.args.push(format!(
            "-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={}",
            lib_dir.display()
        ));
        if !self.profile.is_empty() {
            self.args
                .push(format!("-DCMAKE_BUILD_TYPE={}", self.profile));
        }
        if !self.cflags.is_empty() {
            env::set_var("CFLAGS", &self.cflags);
            self.args.push("-UCMAKE_C_FLAGS".to_string());
        }
        if !self.cxxflags.is_empty() {
            env::set_var("CXXFLAGS", &self.cflags);
            self.args.push("-UCMAKE_CXX_FLAGS".to_string());
        }
        cmd.args(&self.args);
        // println!("cargo:warning={:?} {:?}", cmd.get_program(), cmd.get_args());
        cmd.status()?;

        // build
        let mut cmd = Command::new("cmake");
        let args = [
            "--build".to_string(),
            bin_dir.display().to_string(),
            "--parallel".to_string(),
            "--config".to_string(),
            self.profile.clone(),
        ]
        .to_vec();
        cmd.args(&args);
        // println!("cargo:warning={:?} {:?}", cmd.get_program(), cmd.get_args());
        cmd.status()?;

        // help finding libs
        println!("cargo:rustc-link-search={}", lib_dir.display());
        Ok(lib_dir)
    }

    pub fn build(&mut self) -> PathBuf {
        self.try_build().unwrap()
    }
}
