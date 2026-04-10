# Project Guidelines

## Code Style

- Keep changes minimal and consistent with the existing Rust and C++ code in [lib/](../lib/) and [cpp_src/](../cpp_src/).
- Prefer UTF-8 encoded text, and keep comments in English unless a file already uses another language.
- Preserve the current FFI and error-code patterns instead of introducing new abstractions without a clear need.

## Architecture

- This is a Windows-only hybrid desktop app.
- Rust owns the file-exchange logic in [lib/](../lib/); the `exchange` library swaps names, and the `msgbox` library handles reminder dialogs.
- Treat [cpp_src/](../cpp_src/) and [main_example.rs](../main_example.rs) as reference-only unless a change explicitly targets them.
- The C++ side calls into Rust through the exported `exchange()` FFI entry point in [lib/lib.rs](../lib/lib.rs).
- Keep the return-code mapping in [lib/types.rs](../lib/types.rs) stable unless the public contract is changing on purpose.

## Build and Test

- Build with `cargo build`.
- Build release artifacts with `cargo build --release`.
- Build the library with `cargo build --lib`.
- The intended runtime target is Windows only, and changes should stay compatible with both x86 and x64 Windows builds.
- There is no dedicated automated test suite in the repository; use `cargo build` and the module tests in [lib/lib.rs](../lib/lib.rs) as the main validation path.

## Conventions

- Treat path handling as Windows-first. Relative paths are resolved against the current working directory, and invalid UTF-8 is reported as a path error.
- Preserve the existing three-step rename flow and conflict checks in the Rust exchange logic.
- Keep temporary-file behavior and error codes aligned with [lib/types.rs](../lib/types.rs) and [lib/exchange.rs](../lib/exchange.rs).
- Do not introduce new libraries or dependencies unless they are clearly necessary and approved first.
- If a required dependency, icon, image, or other asset is missing, ask before inventing a substitute or adding one.
- Resource generation depends on [build.rs](../build.rs), [res.ico](../res.ico), and [manifest.xml](../manifest.xml); do not assume cross-platform builds.

## Pitfalls

- The project does not build on non-Windows targets because [build.rs](../build.rs) exits early or fails depending on resource availability and Windows-specific assumptions.
- Missing `res.ico` or `manifest.xml` breaks the resource build step.
- The FFI layer in [lib/lib.rs](../lib/lib.rs) accepts raw C pointers, so null or invalid inputs are unsafe by design.
- The rename flow can leave temporary files behind if I/O fails mid-operation.

## Docs

- User-facing usage lives in [README.md](../README.md); link to it instead of duplicating usage details here.
- There are no additional workspace docs such as `docs/**/*.md`, `CONTRIBUTING.md`, or `ARCHITECTURE.md` in this repository.
