#!/bin/bash

set -euo pipefail

# Parse arguments
MODE="local"
if [ "${1:-}" = "bin" ]; then
    MODE="bin"
fi

# Create a temporary directory that will self-destruct
TMP_DIR=$(mktemp -d -t keyboard-middleware-install.XXXXXX)
START_DIR=$(pwd)
trap 'cd "$START_DIR" && rm -rf "$TMP_DIR"' EXIT INT TERM

# Verify we're on Arch Linux
if [ ! -f /etc/arch-release ]; then
    echo "This script only supports Arch Linux."
    echo "For other distros, download the precompiled binary:"
    echo "  https://github.com/fibsussy/keyboard-middleware/releases/latest"
    exit 1
fi

if [ "$MODE" = "bin" ]; then
    echo "Installing precompiled binary..."
    # Download PKGBUILD.bin and install script
    curl -fsSL -o "$TMP_DIR/PKGBUILD" "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/PKGBUILD.bin"
    curl -fsSL -o "$TMP_DIR/keyboard-middleware.install" "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/keyboard-middleware.install"

    cd "$TMP_DIR"
    makepkg -si --noconfirm
    echo "keyboard-middleware installed successfully via pacman (precompiled binary)"
else
    echo "Building from source..."
    # Download PKGBUILD and install script
    curl -fsSL -o "$TMP_DIR/PKGBUILD" "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/PKGBUILD"
    curl -fsSL -o "$TMP_DIR/keyboard-middleware.install" "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/keyboard-middleware.install"

    cd "$TMP_DIR"
    makepkg -si --noconfirm
    echo "keyboard-middleware installed successfully via pacman (built from source)"
fi
