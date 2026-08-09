use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CARLA_ROOT: &str = "../../vendor/Carla";

fn main() {
    emit_rerun_directives();

    let target_os =
        env::var("CARGO_CFG_TARGET_OS").expect("Cargo did not provide CARGO_CFG_TARGET_OS");
    if target_os != "linux" {
        panic!("carla-sys currently supports Linux targets only; target OS was {target_os}");
    }

    let carla_root = Path::new(CARLA_ROOT);
    let cmake_project = carla_root.join("cmake/CMakeLists.txt");
    if !cmake_project.is_file() {
        panic!(
            "Carla submodule is not initialized; run `git submodule update --init vendor/Carla`"
        );
    }
    let carla_root = carla_root
        .canonicalize()
        .expect("failed to resolve the Carla submodule path");

    require_cmake();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"));
    let library_dir = out_dir.join("native");
    let library_dir_string = library_dir.to_string_lossy().into_owned();

    let mut config = cmake::Config::new(carla_root.join("cmake"));
    config
        .build_target("carla-standalone")
        .define("CARLA_BUILD_STATIC", "OFF")
        .define("CARLA_USE_JACK", "OFF")
        .define("CARLA_USE_OSC", "OFF")
        .define("CMAKE_DISABLE_FIND_PACKAGE_PkgConfig", "ON")
        .define("CMAKE_LIBRARY_OUTPUT_DIRECTORY", &library_dir_string)
        .define("CMAKE_ARCHIVE_OUTPUT_DIRECTORY", &library_dir_string)
        .define("CMAKE_RUNTIME_OUTPUT_DIRECTORY", &library_dir_string);

    for configuration in ["DEBUG", "RELEASE", "RELWITHDEBINFO", "MINSIZEREL"] {
        config.define(
            format!("CMAKE_LIBRARY_OUTPUT_DIRECTORY_{configuration}"),
            &library_dir_string,
        );
        config.define(
            format!("CMAKE_ARCHIVE_OUTPUT_DIRECTORY_{configuration}"),
            &library_dir_string,
        );
        config.define(
            format!("CMAKE_RUNTIME_OUTPUT_DIRECTORY_{configuration}"),
            &library_dir_string,
        );
    }

    config.build();

    let backend = library_dir.join("libcarla_standalone2.so");
    if !backend.is_file() {
        panic!(
            "Carla CMake build completed without producing {}",
            backend.display()
        );
    }

    generate_bindings(&carla_root, &out_dir);

    println!("cargo::rustc-link-search=native={}", library_dir.display());
    println!("cargo::rustc-link-lib=dylib=carla_standalone2");
    println!(
        "cargo::metadata=include_backend={}",
        carla_root.join("source/backend").display()
    );
    println!(
        "cargo::metadata=include_includes={}",
        carla_root.join("source/includes").display()
    );
    println!(
        "cargo::metadata=include_utils={}",
        carla_root.join("source/utils").display()
    );
    println!("cargo::metadata=library_dir={}", library_dir.display());
}

fn generate_bindings(carla_root: &Path, out_dir: &Path) {
    let backend_include = carla_root.join("source/backend");
    let public_include = carla_root.join("source/includes");
    let utils_include = carla_root.join("source/utils");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c")
        .clang_arg(format!("-I{}", backend_include.display()))
        .clang_arg(format!("-I{}", public_include.display()))
        .clang_arg(format!("-I{}", utils_include.display()))
        .allowlist_function("carla_.*")
        .allowlist_type(
            "(Binary|Plugin|Parameter|Midi|Custom|Engine|Nsm|File|Patchbay|Internal|Special|Carla).*",
        )
        .allowlist_var(
            "(BINARY|PLUGIN|PARAMETER|MIDI|CUSTOM|ENGINE|NSM|FILE|PATCHBAY|CONTROL|CARLA)_.*",
        )
        .derive_default(false)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate Carla host bindings; ensure libclang is installed");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write generated Carla host bindings to OUT_DIR");
}

fn require_cmake() {
    let cmake = env::var_os("CMAKE").unwrap_or_else(|| "cmake".into());
    let available = Command::new(&cmake)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());

    if !available {
        panic!(
            "CMake is required to build Carla, but `{}` could not be executed",
            Path::new(&cmake).display()
        );
    }
}

fn emit_rerun_directives() {
    for path in [
        "build.rs",
        "wrapper.h",
        "../../vendor/Carla/cmake/CMakeLists.txt",
        "../../vendor/Carla/source/backend",
        "../../vendor/Carla/source/includes",
        "../../vendor/Carla/source/modules",
        "../../vendor/Carla/source/native-plugins",
        "../../vendor/Carla/source/utils",
        "../../.git/modules/vendor/Carla/HEAD",
    ] {
        println!("cargo::rerun-if-changed={path}");
    }

    for variable in [
        "CMAKE",
        "CMAKE_GENERATOR",
        "CMAKE_PREFIX_PATH",
        "CMAKE_TOOLCHAIN_FILE",
        "CC",
        "CFLAGS",
        "CXX",
        "CXXFLAGS",
    ] {
        println!("cargo::rerun-if-env-changed={variable}");
    }
}
