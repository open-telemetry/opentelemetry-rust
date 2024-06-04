#!/bin/bash

set -eu

# Check if a version is specified as parameter
if [ $# -eq 0 ]; then
  echo "No Rust version specified. Usage: $0 <rust-version>"
  exit 1
fi

RUST_VERSION=$1

# Determine the directory containing the script
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

# Path to the configuration file
CONFIG_FILE="$SCRIPT_DIR/msrv_config.json"

# Change to the root directory of the repository
cd "$SCRIPT_DIR/.."

echo "Current working directory: $(pwd)"

# function to check if specified toolchain is installed
check_rust_toolchain_installed() {
  local version=$1
  if ! rustup toolchain list | grep -q "$version"; then
    echo "Rust toolchain $version is not installed. Please install it using 'rustup toolchain install $version'."
    exit 1
  fi
}

# check if specified toolchain is installed
check_rust_toolchain_installed "$RUST_VERSION"

# Extract the exact installed rust version string
installed_version=$(rustup toolchain list | grep "$RUST_VERSION" | awk '{print $1}')

# Read the configuration file and get the packages for the specified version
if [ -f "$CONFIG_FILE" ]; then
  packages=$(jq -r --arg version "$RUST_VERSION" '.[$version] | .[]' "$CONFIG_FILE" | tr '\n' ' ')
  if [ -z "$packages" ]; then
    echo "No packages found for Rust version $RUST_VERSION in the configuration file."
    exit 1
  fi
else
  echo "Configuration file $CONFIG_FILE not found."
  exit 1
fi

# Check MSRV for the packages
for package in $packages; do
  package=$(echo "$package" | tr -d '\r\n') # Remove any newline and carriage return characters
  echo "Command: rustup run \"$installed_version\" cargo check --manifest-path=\"$package\" --all-features"
  rustup run "$installed_version" cargo check --manifest-path=$package --all-features
done
