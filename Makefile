build:
	cargo build --release --examples --features="bevy/dynamic"

check:
	cargo check --release --examples --features="bevy/dynamic"

doc:
	cargo doc --open --workspace --release --features="bevy/dynamic"