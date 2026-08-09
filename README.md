# rust-carla-rack

A Rust workspace for building an audio-plugin rack on Carla's backend library. The current workspace contains only `carla-sys`, the raw FFI foundation. It does not build or use Carla's Qt/Python frontend and does not yet provide a safe Rust host API.

## Checkout

Clone with submodules:

```sh
git clone --recurse-submodules <repository-url>
cd rust-carla-rack
```

For an existing checkout, initialize the pinned Carla source with:

```sh
git submodule update --init vendor/Carla
```

The backend build does not require Carla's nested `Carla-Plugins` submodule.

## Prerequisites

CI builds and tests native x64 and ARM64 targets on Linux, Windows, and macOS. Building requires:

- Rust and Cargo 1.85 or newer;
- Git;
- CMake 3.15 or newer;
- C and C++ compilers with C++11 support;
- Clang and libclang for bindgen.

Carla's optional pkg-config dependencies are not needed by this build. Check the installed toolchain before building:

```sh
rustc --version
cargo --version
git --version
cmake --version
cc --version
c++ --version
clang --version
```

Bindgen will report an actionable build error if libclang is not available.

## Build and test

Run all project commands at the repository root:

```sh
cargo build --workspace
cargo test --workspace --all-targets
cargo build --workspace --release
```

Cargo invokes CMake through `carla-sys`'s build script. Native artifacts and generated bindings are written below Cargo's `target` directory; no separate CMake or Make command is needed.

The initial build creates Carla's shared `carla_standalone2` backend with JACK, OSC, and ambient pkg-config integrations disabled. Carla's internal plugin-format support, including JSFX, remains enabled. The smoke test uses Carla's `Dummy` driver and needs no audio device, JACK server, plugin installation, display server, or network access.

Cargo supplies the native library search path while it runs workspace build products and tests. Packaging or running a copied downstream executable independently will also require distributing Carla's shared backend (`libcarla_standalone2.so`, `libcarla_standalone2.dylib`, or `libcarla_standalone2.dll`) and configuring the platform's runtime library search path; that deployment policy is outside this milestone.

## Updating Carla

`vendor/Carla` is pinned to a reviewed commit. Update it deliberately rather than following an upstream branch: inspect CMake and C API changes, move the submodule gitlink, regenerate through Cargo, and run the complete workspace test suite before committing the update.

## Licensing

Carla and its standalone backend are licensed under GPL-2.0-or-later. This workspace and `carla-sys` use the same package license because they distribute generated declarations from Carla's GPL headers and link the bundled backend. Upstream notices and license texts remain in `vendor/Carla`, including `vendor/Carla/doc/GPL.txt`.
