# https://github.com/messense/homebrew-macos-cross-toolchains
# This script is used to install cross-compilation toolchains on macOS
# and set up the cross-compilation environment for cargo.
#
# Tested on macOS Ventura 13.4 (22F66) (Apple M1 chip) with Homebrew 4.0.20

brew tap messense/macOS-cross-toolchains

# Install cross-compilation toolchains
brew install x86_64-unknown-linux-gnu
brew install aarch64-unknown-linux-gnu
brew install x86_64-unknown-linux-musl
brew install aarch64-unknown-linux-musl

# Set up the cross-compilation environment
OPENSSL_DIR="$(brew --prefix openssl@1.1)"
export OPENSSL_DIR
# x86_64-unknown-linux-gnu
export CC_X86_64_UNKNOWN_LINUX_GNU=x86_64-unknown-linux-gnu-gcc
export CXX_X86_64_UNKNOWN_LINUX_GNU=x86_64-unknown-linux-gnu-g++
export AR_X86_64_UNKNOWN_LINUX_GNU=x86_64-unknown-linux-gnu-ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc
# aarch64-unknown-linux-gnu
export CC_AARCH64_UNKNOWN_LINUX_GNU=aarch64-unknown-linux-gnu-gcc
export CXX_AARCH64_UNKNOWN_LINUX_GNU=aarch64-unknown-linux-gnu-g++
export AR_AARCH64_UNKNOWN_LINUX_GNU=aarch64-unknown-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-unknown-linux-gnu-gcc
# x86_64-unknown-linux-musl
export CC_X86_64_UNKNOWN_LINUX_MUSL=x86_64-unknown-linux-musl-gcc
export CXX_X86_64_UNKNOWN_LINUX_MUSL=x86_64-unknown-linux-musl-g++
export AR_X86_64_UNKNOWN_LINUX_MUSL=x86_64-unknown-linux-musl-ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc
# aarch64-unknown-linux-musl
export CC_AARCH64_UNKNOWN_LINUX_MUSL=aarch64-unknown-linux-musl-gcc
export CXX_AARCH64_UNKNOWN_LINUX_MUSL=aarch64-unknown-linux-musl-g++
export AR_AARCH64_UNKNOWN_LINUX_MUSL=aarch64-unknown-linux-musl-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-unknown-linux-musl-gcc

# Install targets
# macOS
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
# linux
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
# Temporary fix: https://github.com/rust-lang/rust/issues/89626#issuecomment-946434059
rustup +nightly target add aarch64-unknown-linux-musl

# Build targets
# macOS
cargo build --release --target=x86_64-apple-darwin
cargo build --release --target=aarch64-apple-darwin
# Linux
cargo build --release --target=x86_64-unknown-linux-gnu
cargo build --release --target=aarch64-unknown-linux-gnu
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
# Temporary fix: https://github.com/rust-lang/rust/issues/89626#issuecomment-946434059
TARGET_CC=aarch64-linux-musl-gcc RUSTFLAGS="-Zgcc-ld=lld" cargo +nightly build --target aarch64-unknown-linux-musl --release


# Find Cargo.toml
# Search for Cargo.toml in the current directory and its parent directories until found
for (( i = 0; i < 3; i++ )); do
    if [ -f Cargo.toml ]; then
        break
    fi
    cd ..
done
# Get the crate name from Cargo.toml
CRATE_NAME=$(grep -m 1 name Cargo.toml | cut -d '"' -f 2)
# Universal macOS binary
mkdir -p target/release/universal-apple-darwin
lipo -create \
    target/x86_64-apple-darwin/release/"$CRATE_NAME" \
    target/aarch64-apple-darwin/release/"$CRATE_NAME" \
    -output target/release/universal-apple-darwin/"$CRATE_NAME"

# Archive
# latest version using git tag or Cargo.toml
VERSION=$(git describe --tags --abbrev=0) || VERSION=$(grep -m 1 version Cargo.toml | cut -d '"' -f 2)
mkdir -p target/release/archive/"$VERSION"
tar -czvf target/release/archive/"$VERSION"/"$CRATE_NAME"-universal-apple-darwin.tar.gz target/release/universal-apple-darwin/"$CRATE_NAME"
tar -czvf target/release/archive/"$VERSION"/"$CRATE_NAME"-x86_64-unknown-linux-gnu.tar.gz target/x86_64-unknown-linux-gnu/release/"$CRATE_NAME"
tar -czvf target/release/archive/"$VERSION"/"$CRATE_NAME"-aarch64-unknown-linux-gnu.tar.gz target/aarch64-unknown-linux-gnu/release/"$CRATE_NAME"
tar -czvf target/release/archive/"$VERSION"/"$CRATE_NAME"-x86_64-unknown-linux-musl.tar.gz target/x86_64-unknown-linux-musl/release/"$CRATE_NAME"
tar -czvf target/release/archive/"$VERSION"/"$CRATE_NAME"-aarch64-unknown-linux-musl.tar.gz target/aarch64-unknown-linux-musl/release/"$CRATE_NAME"






