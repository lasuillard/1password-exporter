#!/usr/bin/env bash

ARCH="$(dpkg --print-architecture)"

CARGO_BINSTALL_VERSION="1.8.0"

# 1Password
curl -sS https://downloads.1password.com/linux/keys/1password.asc | sudo gpg --dearmor --output /usr/share/keyrings/1password-archive-keyring.gpg \
  && echo "deb [arch=${ARCH} signed-by=/usr/share/keyrings/1password-archive-keyring.gpg] https://downloads.1password.com/linux/debian/${ARCH} stable main" | sudo tee /etc/apt/sources.list.d/1password.list

sudo apt-get update && sudo apt-get install -y \
  1password-cli

curl -fsSL "https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-$(rustc -vV | sed -n 's|host: ||p').tgz" \
    | tar --extract --gzip --directory "${CARGO_HOME}/bin"

# Download dev tools binaries
cargo binstall -y --log-level debug \
    cargo-llvm-cov \
    cargo-nextest \
    cargo-udeps \
    cargo-watch
