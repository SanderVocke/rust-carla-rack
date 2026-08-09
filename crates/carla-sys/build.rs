use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CARLA_ROOT: &str = "../../vendor/Carla";

fn main() {
    emit_rerun_directives();

    let target_os =
        env::var("CARGO_CFG_TARGET_OS").expect("Cargo did not provide CARGO_CFG_TARGET_OS");
    if !matches!(target_os.as_str(), "linux" | "macos" | "windows") {
        panic!("carla-sys does not support target OS `{target_os}`");
    }
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo did not provide CARGO_CFG_TARGET_ARCH");

    let carla_root = Path::new(CARLA_ROOT);
    let cmake_project = carla_root.join("cmake/CMakeLists.txt");
    if !cmake_project.is_file() {
        panic!(
            "Carla submodule is not initialized; run `git submodule update --init vendor/Carla`"
        );
    }
    let carla_root = normalize_canonical_path(
        carla_root
            .canonicalize()
            .expect("failed to resolve the Carla submodule path"),
    );

    require_cmake();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"));
    let library_dir = out_dir.join("native");
    let library_dir_string = library_dir.to_string_lossy().into_owned();

    let windows_arm64 = target_os == "windows" && target_arch == "aarch64";
    let cmake_source = if windows_arm64 {
        PathBuf::from("cmake")
    } else {
        carla_root.join("cmake")
    };
    let mut config = cmake::Config::new(cmake_source);
    if windows_arm64 {
        config.define("CARLA_SOURCE_DIR", carla_root.as_os_str());
    }
    config
        .build_target("carla-standalone")
        .define("CARLA_BUILD_FRAMEWORKS", "OFF")
        .define("CARLA_BUILD_STATIC", "OFF")
        .define("CARLA_USE_JACK", "OFF")
        .define("CARLA_USE_OSC", "OFF")
        .define("CMAKE_DISABLE_FIND_PACKAGE_PkgConfig", "ON")
        .define("CMAKE_LIBRARY_OUTPUT_DIRECTORY", &library_dir_string)
        .define("CMAKE_ARCHIVE_OUTPUT_DIRECTORY", &library_dir_string)
        .define("CMAKE_RUNTIME_OUTPUT_DIRECTORY", &library_dir_string);

    configure_platform(&mut config, &target_os, &target_arch);
    configure_compiler_cache(&mut config);

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

    let backend_name = match target_os.as_str() {
        "linux" => "libcarla_standalone2.so",
        "macos" => "libcarla_standalone2.dylib",
        "windows" => "libcarla_standalone2.dll",
        _ => unreachable!(),
    };
    let backend = library_dir.join(backend_name);
    if !backend.is_file() {
        panic!(
            "Carla CMake build completed without producing {}",
            backend.display()
        );
    }

    generate_bindings(&carla_root, &out_dir);

    let link_name = if target_os == "windows" {
        "libcarla_standalone2"
    } else {
        "carla_standalone2"
    };
    println!("cargo::rustc-link-search=native={}", library_dir.display());
    println!("cargo::rustc-link-lib=dylib={link_name}");
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

fn normalize_canonical_path(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        let display = path.to_string_lossy();
        if let Some(path) = display.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{path}"));
        }
        if let Some(path) = display.strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
    }

    path
}

fn configure_platform(config: &mut cmake::Config, target_os: &str, target_arch: &str) {
    if target_os == "macos" {
        let cmake_arch = match target_arch {
            "aarch64" => "arm64",
            "x86_64" => "x86_64",
            _ => panic!("carla-sys does not support macOS target architecture `{target_arch}`"),
        };
        config
            .define("CMAKE_OSX_ARCHITECTURES", cmake_arch)
            .define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0");
    }

    if target_os == "windows" {
        config
            .define("CMAKE_POLICY_DEFAULT_CMP0141", "NEW")
            .define("CMAKE_MSVC_DEBUG_INFORMATION_FORMAT", "Embedded");
    }
}

fn configure_compiler_cache(config: &mut cmake::Config) {
    for variable in ["CMAKE_C_COMPILER_LAUNCHER", "CMAKE_CXX_COMPILER_LAUNCHER"] {
        if let Some(value) = env::var_os(variable) {
            config.define(variable, value);
        }
    }
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
        "cmake/CMakeLists.txt",
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
        "CMAKE_C_COMPILER_LAUNCHER",
        "CMAKE_CXX_COMPILER_LAUNCHER",
        "CC",
        "CFLAGS",
        "CXX",
        "CXXFLAGS",
    ] {
        println!("cargo::rerun-if-env-changed={variable}");
    }
}
