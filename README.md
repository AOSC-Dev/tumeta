tumeta
======

Topic update manifest converter for AOSC OS, see <https://gist.github.com/MingcongBai/912e778216aad58cf504713dcd4898cc> for more details.

CLI Tool
========

```bash
cargo build -p tumeta --release  # Build the CLI tool
cargo run -p tumeta --release -- --src <path to manifests in TOML format> --dst <path to topic.json>  # With cargo run
tumeta --src <path to manifests in TOML format> --dst <path to topic.json>  # Run tumeta binary directly
```

```
Usage: tumeta [OPTIONS] --src <SRC> --dst <DST>

Options:
  -s, --src <SRC>     Path to source file(s) in TOML format
  -d, --dst <DST>     Path to destination JSON file
  -i, --ignore-error  Ignore errors
  -h, --help          Print help
  -V, --version       Print version
```

Rust Library
============

Add `topic_meta`(topic_meta) to `Cargo.toml`:

```toml
topic_meta = { git = "https://github.com/AOSC-Dev/tumeta.git" }
```

Run `cargo doc --open` for API docs.
