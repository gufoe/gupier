
set -e
cargo build --release --target x86_64-unknown-linux-musl
ls -l target/x86_64-unknown-linux-musl/release/gupier
strip -s target/x86_64-unknown-linux-musl/release/gupier
ls -l target/x86_64-unknown-linux-musl/release/gupier
upx target/x86_64-unknown-linux-musl/release/gupier
