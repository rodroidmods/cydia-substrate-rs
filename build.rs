use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let substrate_src = manifest_dir.join("substrate");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-parentheses")
        .include(&substrate_src);

    let sources = vec![
        substrate_src.join("SubstrateHook.cpp"),
        substrate_src.join("SubstrateDebug.cpp"),
        substrate_src.join("SubstratePosixMemory.cpp"),
        substrate_src.join("SubstrateStubs.cpp"),
    ];

    for source in &sources {
        build.file(source);
        println!("cargo:rerun-if-changed={}", source.display());
    }

    match target_os.as_str() {
        "android" => {
            build
                .file(substrate_src.join("SymbolFinder.cpp"))
                .define("__ANDROID__", None);

            println!("cargo:rustc-link-lib=log");
            println!("cargo:rustc-link-lib=android");
        }
        "ios" => {
            build.define("__APPLE__", None);
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=UIKit");
        }
        _ => {}
    }

    match target_arch.as_str() {
        "x86_64" | "x86" => {
            build.file(substrate_src.join("hde64.c"));
        }
        "aarch64" => {
            if target_os == "android" {
                build.file(substrate_src.join("And64InlineHook.cpp"));
            }
        }
        "arm" => {}
        _ => {}
    }

    if cfg!(feature = "debug") {
        build.define("MSDEBUG", "1");
    }

    build.compile("substrate");

    generate_bindings(&substrate_src, &out_path, &target_os, &target_arch);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", substrate_src.display());
}

fn generate_bindings(substrate_src: &PathBuf, out_path: &PathBuf, target_os: &str, target_arch: &str) {
    let mut builder = bindgen::Builder::default()
        .header(substrate_src.join("CydiaSubstrate.h").to_str().unwrap())
        .clang_arg(format!("-I{}", substrate_src.display()))
        .clang_arg("-xc++")
        .clang_arg("-std=c++11")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("MSHookFunction")
        .allowlist_function("MSHookProcess")
        .allowlist_function("MSGetImageByName")
        .allowlist_function("MSFindSymbol")
        .allowlist_type("MSImageRef")
        .allowlist_var("MSDebug")
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true);

    if target_os == "ios" {
        builder = builder
            .allowlist_function("MSHookMessageEx")
            .clang_arg("-D__APPLE__");
    }

    if target_os == "android" {
        builder = builder
            .header(substrate_src.join("SymbolFinder.h").to_str().unwrap())
            .allowlist_function("find_name")
            .allowlist_function("find_libbase")
            .clang_arg("-D__ANDROID__");
    }

    if target_arch == "x86_64" || target_arch == "x86" {
        builder = builder
            .header(substrate_src.join("hde64.h").to_str().unwrap())
            .allowlist_function("hde64_disasm")
            .allowlist_type("hde64s");
    }

    if target_arch == "aarch64" && target_os == "android" {
        builder = builder
            .header(substrate_src.join("And64InlineHook.hpp").to_str().unwrap())
            .allowlist_function("A64HookFunction")
            .allowlist_function("A64HookFunctionV");
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
