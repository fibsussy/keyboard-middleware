#!/bin/bash
set -euo pipefail

show_help() {
    cat <<EOF_HELP
Keyboard Middleware Installer

Usage: $0 [OPTION] [VERSION]

Options:
  local     Build local WIP version (auto-detected with uncommitted changes)
  git       Build from git source
  bin       Install precompiled binary (default behavior)
  -v, --version VERSION  Install specific git tag/version
  --help    Show this help message

Examples:
  $0                  # Auto-detect: WIP if dirty, git if clean repo, bin if not a repo
  $0 local             # Force local WIP build (only works with uncommitted changes)
  $0 git               # Build from git source
  $0 bin               # Install latest binary
  $0 -v v1.2.0         # Install version v1.2.0 from git
  $0 bin -v v1.2.0     # Install version v1.2.0 binary
EOF_HELP
    exit 0
}

MODE=""
VERSION=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        local)
            MODE="local"
            shift
            ;;
        bin)
            MODE="bin"
            shift
            ;;
        git)
            MODE="git"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

START_DIR=$(pwd)

# Validate incompatible options
if [ -n "$VERSION" ] && [ "$MODE" = "local" ]; then
    echo "Error: --version is incompatible with local mode (local builds are always WIP)"
    exit 1
fi

# Atomic build in subshell that self-destructs
build_and_install() {
    (
        # Create temp directory that gets destroyed when subshell exits
        cd "$(mktemp -d)"
        
        # Set up cleanup trap for this subshell
        trap 'rm -rf "$PWD"' EXIT
        
        echo "Working in temporary directory: $PWD"
        
        # Copy/source files from original directory
        if [ -f "$START_DIR/PKGBUILD" ] && [ -f "$START_DIR/Cargo.toml" ] && [ -z "$VERSION" ]; then
            echo "Detected keymux repository..."
            
            # Auto-detect mode if not specified
            if [ -z "$MODE" ]; then
                if ! git -C "$START_DIR" diff --quiet || ! git -C "$START_DIR" diff --cached --quiet 2>/dev/null; then
                    echo "Found uncommitted changes, using local WIP build"
                    MODE="local"
                else
                    echo "Repository is clean, using git build"
                    MODE="git"
                fi
            fi
            
            if [ "$MODE" = "bin" ]; then
                echo "Installing binary package..."
                cp "$START_DIR/PKGBUILD-bin" ./PKGBUILD
                cp "$START_DIR/keymux.install" ./
                makepkg -si
            elif [ "$MODE" = "local" ]; then
                echo "Building local WIP package..."
            elif [ "$MODE" = "git" ]; then
                echo "Building git package..."
                cp "$START_DIR/PKGBUILD-git" ./PKGBUILD
                cp "$START_DIR/keymux.install" ./
                makepkg -si
            fi

        else
            echo "Installing from remote repository..."
            [ -z "$MODE" ] && MODE="bin"
            
            if [ "$MODE" = "bin" ]; then
                echo "Installing binary package..."
                if [ -n "$VERSION" ]; then
                    curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keymux/main/PKGBUILD-bin"
                    sed -i "s/pkgver=.*/pkgver=${VERSION#v}/" PKGBUILD
                else
                    curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keymux/main/PKGBUILD-bin"
                fi
                curl -fsSL -o keymux.install "https://raw.githubusercontent.com/fibsussy/keymux/main/keymux.install"
            else
                echo "Building git package..."
                curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keymux/main/PKGBUILD-git"
                curl -fsSL -o keymux.install "https://raw.githubusercontent.com/fibsussy/keymux/main/keymux.install"
            fi
            
            makepkg -si
        fi
    )
}

# Run the atomic build
build_and_install

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Installation complete! To enable the services:"
echo ""
echo "  Root daemon (required):"
echo "    sudo systemctl enable --now keymux.service"
echo ""
echo "  Niri watcher (optional, for auto game mode):"
echo "    systemctl --user enable --now keymux-niri.service"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
