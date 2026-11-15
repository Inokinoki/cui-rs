# Changelog — `cui-rs-macros`

## Unreleased

- First release: the `view!` macro, a declarative front end for the
  per-frame call sequence of `cui-rs`. Publish this crate before `cui-rs`.
- `#![containers(..)]` header for custom scope widgets; opts types resolve in the caller's scope when `cui::view` has no match.
- `src/generated.rs` (written by `scripts/gen_ffi.py`): the generated scope widgets and the leaf widgets that always go through `name_with`.
