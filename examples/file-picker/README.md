# File Picker Example

Demonstrates native file picker dialogs with the `file-picker` feature.

## What it demonstrates

Opening native file/folder pickers and save dialogs, with results displayed as toasts.

## Concepts exercised

- `pick_file`, `pick_files`, `pick_folder`, `save_file` functions
- `PickFileParams` and `SaveFileParams` configuration
- `FileFilter` for filtering file types
- `perform` for async picker results
- `Toast::success` for displaying results
- Feature-gated via `nive = { features = ["file-picker"] }`

## How to run

```bash
cd examples/file-picker
cargo run
```
