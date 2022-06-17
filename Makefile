build:
	cargo build --release --examples --features="bevy/dynamic"

doc:
	cargo doc --open --workspace --release