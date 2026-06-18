# Devtools Contract

Devtools is an optional `nive-runtime` capability.

```toml
[features]
default = []
devtools = ["dep:nive-runtime-derive"]
```

With the feature enabled, `nive-runtime` reexports the root `Devtools` derive.
The existing application-specific derives remain available during the staged
migration. Runtime host ownership and removal of the direct app derive
dependency occur in the dedicated Devtools and facade-cleanup slices.
