# nive-workbench

`nive-workbench` provides a fixed-region professional desktop shell built from
`nive-ui` primitives. It covers document tabs, generic panel hosts, side rails,
bottom header tabs, layout/session state, diagnostics/status surfaces, and
command palette hosting.

The crate stores shell/view state only. Application domain state, side effects,
persistence location, runtime/window behavior, resources, operations, and final
message routing remain app-owned.

Runtime adapters are available behind the optional `runtime` feature.
