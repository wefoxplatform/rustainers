#!/bin/sh

set -e
# set -x

export DEBIAN_FRONTEND=noninteractive

# Install dependencies
apt-get update -y -qq
apt-get install -y -qq \
  bash \
  curl \
  unzip \
  time \
  gettext \
  ca-certificates \
  gnupg \
  lsb-release

# Install docker
# from https://docs.docker.com/engine/install/debian/
#shellcheck disable=SC2174
mkdir -m 0755 -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list >/dev/null
apt-get update -y -qq
apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
# docker --version

# Clean up
time rm -rf /var/lib/apt/lists

# Install Rust complementary tool
rustup --version
rustup toolchain list
rustup component add clippy rustfmt

cargo --version
cargo clippy --version
cargo fmt --version
