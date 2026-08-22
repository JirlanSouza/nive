# Async actions

Shows the reducer-friendly tracked-request tier. The reducer owns only UI
state, mints affine request handles, and hands them to an application-owned
service runner. It also demonstrates typed operation output, drop-new
admission, explicit cancellation, and a child `TaskScope` owned by an optional
screen. Closing the screen drops its scope; a later cancellation message is
routed as a safe no-op because the screen state is already absent.

Run from the repository root:

```bash
just example-dev async-actions
```
