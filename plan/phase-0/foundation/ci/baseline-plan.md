# CI Baseline Plan v0.1

Status: Accepted

Required checks:

1. formatting
```bash
cargo fmt --all --check
```

2. lint
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

3. build
```bash
cargo build --workspace --locked
```

4. test
```bash
cargo test --workspace --all-features
```

5. SQLx metadata validation
```bash
cargo sqlx prepare --check
```

Multi-arch baseline:
- `linux/amd64`
- `linux/arm64`
