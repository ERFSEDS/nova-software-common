# How to build this

This is the flight code, so it has to be uploaded to the flight computer board. This is done using a cargo tool named **probe-run** ([github](https://github.com/knurling-rs/probe-run)). 

This tool can be installed by running the following command:

```bash
cargo install probe-run
```

When you want to build or upload the code, you can then just run:

```bash
cargo build
```

or

```bash
cargo run
```

## Rust toolchain

The Rust target for the particular board must be installed in order for Rust to know how to compile for it. The command to install the current board's target is:

```
rustup target add thumbv7em-none-eabihf
```