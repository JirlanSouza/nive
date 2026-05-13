# List configured app icons.
icons-list:
    cargo run --quiet --package xtask -- icons list

# Sync vendored Lucide SVG assets and generated Rust mapping.
icons-sync:
    cargo run --quiet --package xtask -- icons sync

# Check that vendored Lucide SVG assets and generated Rust mapping are current.
icons-check:
    cargo run --quiet --package xtask -- icons check

# Add an app icon mapping and sync assets. Usage: just icons-add Search search
icons-add variant lucide_name:
    cargo run --quiet --package xtask -- icons add {{ variant }} {{ lucide_name }}
