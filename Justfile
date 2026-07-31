# Justfile for the Wayle workspace
# -------------------------------------------------
# Default task
#   just         -> builds in debug mode (alias for `just debug`)
#   just debug   -> cargo build (debug)
#   just release -> cargo build --release (production)
#   just run     -> cargo run (debug)
#   just run-release -> cargo run --release
#   just test    -> run all tests
#   just clean   -> clean the workspace
# -------------------------------------------------

# Default target builds the debug version
default := "run"

debug:
	@echo "Building debug version..."
	cargo build --profile dev

release:
	@echo "Building release (production) version..."
	cargo build --release

run:
	@echo "Running debug executable..."
	cargo run --profile dev --bin=wayle shell

run-settings:
	@echo "Running debug settings executable..."
	cargo run --profile dev --bin=wayle-settings

run-release:
	@echo "Running release executable..."
	cargo run --release

test:
	@echo "Running tests..."
	cargo test

clean:
	@echo "Cleaning workspace..."
	cargo clean
