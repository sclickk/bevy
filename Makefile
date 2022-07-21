build:
	cargo build --release --examples --all-features

check:
	cargo check --release --examples --all-features

doc:
	cargo doc --open --workspace --release --all-features