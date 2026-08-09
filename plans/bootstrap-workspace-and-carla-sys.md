# Bootstrap the workspace and `carla-sys`

## Goals and scope

Create the initial virtual Cargo workspace and one deliberately thin `carla-sys` crate. The crate will own a pinned Carla source submodule, build Carla's standalone backend as part of Cargo's build graph, generate raw Rust FFI declarations from the public C host API, and link consumers to that backend.

This milestone includes:

- the top-level workspace and shared package policy;
- a pinned `falkTX/Carla` Git submodule;
- Cargo-driven CMake compilation of `carla-standalone` only;
- raw bindings to `CarlaHost.h` and a backend lifecycle smoke test;
- build and checkout documentation.

It excludes safe wrappers, rack/graph abstractions, plugin discovery policy, Carla's Qt/Python frontend, audio-plugin UI integration, packaging native runtime artifacts for end users, and additional Rust crates.

## Immutable acceptance criteria

1. A recursive checkout is recognized as a Cargo workspace from the repository root, with `crates/carla-sys` as a member and resolver 3 enabled.
2. Carla is present only as a Git submodule, pinned to a reviewed commit; no copied or locally modified Carla source is tracked by the parent repository.
3. `cargo build --workspace` from the repository root configures and builds Carla's `carla-standalone` CMake target and links `carla-sys` to `carla_standalone2`; users do not manually invoke CMake or Make.
4. The Cargo build does not build or depend on Carla's GUI/frontend targets, and native build outputs stay under Cargo's target/`OUT_DIR` tree rather than modifying the submodule.
5. `carla-sys` publicly exposes the raw C host API needed to embed the backend, generated from the pinned `CarlaHost.h`, without adding safe ownership or lifecycle abstractions.
6. On the initially documented Linux build environment, a clean build passes an integration smoke test that creates a standalone host, starts the `Dummy` engine with compatible process/transport modes, closes it, and thereby proves that generated declarations resolve to the built backend.
7. Required native tools, the submodule checkout step, the Cargo-only commands, the initial feature choices, platform validation scope, and Carla's GPL-2.0-or-later licensing implications are documented.

## Investigation basis

- Carla `v2.5.10` is the latest stable release but has no CMake integration. Carla commit `97a9e0740baf6df2df942495c02532a624c44682` identifies itself as `2.6.0-alpha1` and contains the embedding-oriented CMake project under `cmake/`; use that reviewed commit as the initial pin.
- The upstream CMake target is `carla-standalone`, while its non-framework output is named `carla_standalone2`.
- Building that target directly avoids Carla's frontend and avoids the broader CMake `install` target. The output directory can be forced into `OUT_DIR` so the submodule remains read-only.
- A local Linux probe successfully built this target with CMake 4.1.2, JACK/OSC/pkg-config integrations disabled, and no Carla nested submodule initialization. The resulting shared backend successfully initialized and closed its `Dummy` engine when configured with `ENGINE_PROCESS_MODE_CONTINUOUS_RACK` and `ENGINE_TRANSPORT_MODE_INTERNAL`.

## Design rules and constraints

- Use `vendor/Carla` for the submodule so future workspace crates share one native source pin. Record the upstream URL and exact commit in Git metadata and documentation.
- Use a virtual Edition 2024 workspace with resolver 3 and an explicit Rust 1.85 MSRV. Centralize package metadata and dependency versions where Cargo supports workspace inheritance.
- Name the crate `carla-sys`, declare `links = "carla_standalone2"`, and keep its API to generated FFI declarations plus only the lint allowances required by generated C names.
- Drive upstream CMake from `crates/carla-sys/build.rs` with the `cmake` crate. Build `carla-standalone` directly instead of the default `install` target, force library/archive/runtime output directories below `OUT_DIR`, and emit precise Cargo rerun, link-search, link-library, and dependency metadata directives.
- Start with the shared Carla backend. This lets CMake perform the complete C++ link of Carla's internal static components; static distribution is deferred until all transitive native link requirements can be modeled and tested.
- Make the initial backend deterministic and headless by disabling JACK, OSC, and pkg-config-discovered optional libraries. Keep Carla's internally supplied plugin-format support unless a build failure demonstrates that a specific component must be disabled. Future optional backend capabilities belong behind explicit Cargo features rather than ambient host detection.
- Generate bindings into `OUT_DIR` during the build using a minimal wrapper header that includes `CarlaHost.h`. Pin `bindgen`, pass the three upstream public include roots, allowlist the `carla_*` API and its referenced public types/constants, and use Cargo callbacks so header changes invalidate bindings.
- Use target information supplied by Cargo in the build script. Initially validate Linux; do not claim macOS/Windows support without native build and runtime tests, but avoid unnecessary Linux assumptions in the public Rust declarations.
- Fail early with an actionable message when the Carla submodule or required build tools/libclang are missing. Never initialize or mutate Git submodules from `build.rs`, and never write generated files outside `OUT_DIR`.
- Preserve upstream licensing notices and mark package/workspace metadata consistently with the GPL-2.0-or-later backend being distributed and linked.

## Staged implementation plan

### Stage 1 — Workspace and pinned native source

- [x] Add the top-level `Cargo.toml` virtual workspace, shared package fields, resolver/MSRV policy, and workspace dependency pins.
- [x] Add a root `.gitignore` for Cargo/native build outputs without hiding source, lockfile policy, or submodule state.
- [x] Add `https://github.com/falkTX/Carla.git` at `vendor/Carla` and pin it to the reviewed CMake-capable commit.
- [x] Add the minimal `crates/carla-sys/Cargo.toml` with inherited metadata, its build script declaration, unique native `links` value, and build dependencies.

Verification:

- [x] Run `git submodule status vendor/Carla` and confirm the expected commit with no dirty suffix.
- [x] Run `cargo metadata --no-deps` from the root and confirm the workspace contains exactly `carla-sys`.
- [x] Confirm `git status --short` reports the submodule gitlink and intended bootstrap files, not Carla's contents as ordinary parent-repository files.

Commit this stage as the workspace/submodule foundation.

### Stage 2 — Cargo-owned Carla backend build

Depends on Stage 1.

- [x] Implement `crates/carla-sys/build.rs` to validate the submodule, configure Carla's `cmake/` project under `OUT_DIR`, select the Cargo profile, apply the deterministic headless options, and build only `carla-standalone`.
- [x] Normalize single- and multi-config CMake output paths from Cargo target information, then emit the correct native search path and dynamic link directive for `carla_standalone2`.
- [x] Add narrow `rerun-if-changed` coverage for the wrapper and Carla source/CMake inputs, `rerun-if-env-changed` coverage for supported toolchain controls, and metadata exposing the native include/library roots to immediate future dependents.
- [x] Ensure diagnostic failures distinguish an uninitialized submodule, missing CMake/compiler, unsupported target handling, and native compilation failure.

Verification:

- [x] Run `cargo clean && cargo build -p carla-sys -vv` from the root and confirm Cargo invokes CMake automatically.
- [x] Confirm the backend shared library exists below the package's `OUT_DIR`, exports `carla_standalone_host_init`, and no build output appears inside `vendor/Carla`.
- [x] Inspect the verbose build to confirm the selected target is `carla-standalone`, not Carla's frontend or the broad install/all target.

Commit this stage as the native backend build milestone.

### Stage 3 — Raw generated host bindings

Depends on Stage 2.

- [x] Add a minimal wrapper header and configure bindgen in `build.rs` for C parsing of Carla's public host header and required include directories.
- [x] Restrict generation to the standalone host functions and recursively required public Carla types/constants; derive only traits that are valid for the underlying C declarations.
- [x] Add `src/lib.rs` to include the generated `OUT_DIR` file, document that the surface is raw/unsafe and Carla-owned return pointers must not be freed, and limit naming/dead-code lint exceptions to generated code.
- [x] Verify representative engine, plugin, parameter, transport, and project functions and their referenced structs/enums are public, while C++ implementation APIs are not exposed.

Verification:

- [x] Run `cargo check -p carla-sys` and `cargo doc -p carla-sys --no-deps` with warnings denied where practical.
- [x] Add compile-time usage coverage for representative constants, opaque handles, callbacks, structs, and function signatures to catch an accidentally over-restrictive allowlist.
- [x] Re-run the build without changes and confirm Cargo/CMake/bindgen do not perform avoidable work.

Commit this stage as the raw FFI binding milestone.

### Stage 4 — Linked backend smoke test and documentation

Depends on Stage 3.

- [x] Add one serialized integration smoke test that calls the raw API to create a host, selects continuous-rack processing and internal transport, starts the `Dummy` driver, reports `carla_get_last_error` on failure, and closes the engine on success.
- [x] Keep the test independent of JACK, physical audio devices, plugin installations, GUI/display services, and network access.
- [x] Add concise root/crate documentation covering recursive checkout, CMake/C/C++/libclang prerequisites, root Cargo commands, shared-library runtime behavior, enabled/disabled Carla capabilities, initial Linux validation, submodule update policy, and licensing.

Verification:

- [x] Run `cargo test -p carla-sys --test backend_smoke -- --nocapture` and confirm backend initialization and shutdown succeed.
- [x] Run the documented commands from the repository root in a shell without Carla-specific library-path overrides.
- [x] Confirm the test and docs use only the public C host API and do not introduce a safe wrapper layer.

Commit this stage as the tested/documented sys-crate milestone.

### Stage 5 — Final end-to-end validation

Depends on all prior stages.

- [ ] From a fresh recursive clone on the documented Linux environment, run the documented prerequisite check and `cargo build --workspace` without any manual native build command.
- [ ] Run `cargo fmt --all --check`.
- [ ] Run `cargo check --workspace --all-targets`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`, with any generated-binding exception narrowly scoped and documented.
- [ ] Run `cargo test --workspace --all-targets`.
- [ ] Run `cargo build --workspace --release` to exercise release-profile CMake configuration and linking.
- [ ] Run `git diff --check`, verify `git submodule status`, and confirm all generated/native artifacts are ignored under Cargo's target tree and both the parent repository and Carla submodule are clean.

Commit any validation-only fixes as a final meaningful milestone, then record the commands and results in this plan.

## Execution contract

- Keep the plan updated as work progresses and check off completed items.
- Commit each completed stage or meaningful milestone.
- Implementation steps may be revised when new evidence warrants it.
- Design rules may be revised for a documented, well-supported reason.
- Goals and acceptance criteria must not be changed without explicit user approval.
