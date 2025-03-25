install-cargo-swift:
	cargo install cargo-swift@0.8.1 -f

build:
	cargo swift package --platforms ios --name AcurastBenchmark --xcframework-name AcurastBenchmarkFFI --release --accept-all
