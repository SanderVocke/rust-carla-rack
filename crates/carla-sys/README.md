# carla-sys

Raw Rust bindings to Carla's standalone plugin-host backend.

The crate's build script:

1. builds the pinned `vendor/Carla` source with its embedding-oriented CMake project;
2. generates bindings from `CarlaHost.h` into Cargo's `OUT_DIR` using bindgen;
3. links consumers to the resulting shared `carla_standalone2` library.

Only Linux is currently supported. CMake 3.15+, C/C++ compilers, Clang, and libclang are required. JACK, OSC, and pkg-config-discovered optional integrations are disabled for a deterministic headless build. Carla's GUI/frontend is not built.

This crate intentionally exposes only Carla's unsafe C API. It does not model handle ownership, pointer lifetimes, callbacks, threading, or engine state safely. Carla-owned return pointers must not be freed unless its API explicitly says otherwise.

See the workspace README for checkout, build, runtime-library, submodule-update, and licensing details.
