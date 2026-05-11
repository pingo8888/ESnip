# Rust unit test layout

The files in this directory are mounted from the matching source modules with
`#[cfg(test)] #[path = "..."] mod tests;`.

This keeps the test bodies out of production source files while preserving Rust
unit-test visibility for module-private functions. When adding a new test file,
also add the corresponding `#[path]` module declaration in the source module
that owns the behavior under test.

Example:

```rust
#[cfg(test)]
#[path = "../../tests/store/notes/foo.rs"]
mod tests;
```

Shared note-store fixtures are a special case: `notes/mod.rs` exposes them as
`pub(crate) mod test_utils;` under `#[cfg(test)]`, so sibling note tests can use
`crate::store::notes::test_utils`.
