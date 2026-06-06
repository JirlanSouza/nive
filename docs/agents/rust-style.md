# Rust Style Guidelines

## Formatting

- Run `just fmt` before opening a PR or finishing broad Rust edits.
- Follow default Rust formatting.
- Keep imports grouped as `std`, external crates, `super::`/`self::`/`crate::`.

## Naming And Modules

- Use `snake_case` for files, modules, functions, and tests.
- Follow the `<name>.rs` plus `<name>/` module pattern.
- Keep Rust tests close to the module they cover.

## Ownership

- Service methods should prefer references such as `&str` instead of owned `String` values for IDs when ownership is not required.
- Command or UI boundary layers should act as data owners and pass references down into services.

## Comments

Do not add comments unless a public Rust API truly needs a short doc comment or a complex block benefits from a concise orientation note.
