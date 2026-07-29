# Contributing

Keep changes focused and preserve the data-flow boundary:

`Arduino -> binary protocol -> Rust validation -> typed events -> React`

Before opening a change:

```powershell
npm.cmd run lint
npm.cmd test
npm.cmd run build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Firmware changes must also compile the examples documented in
`firmware/README.md`. Update the protocol specification and shared vectors when
wire bytes change.

