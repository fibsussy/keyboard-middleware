#!/bin/bash
set -euo pipefail

sudo -v

show_help() {
    cat <<EOF_HELP
Keyboard Middleware Installer

Usage: $0 [OPTION] [VERSION]

Options:
  local     Build from source (default if in repo)
  bin       Install precompiled binary
  -v, --version VERSION  Install specific git tag/version
  --help    Show this help message

Examples:
  $0 local            # Build from local source
  $0 bin              # Install latest binary
  $0 -v v1.2.0        # Install version v1.2.0 from source
  $0 bin -v v1.2.0    # Install version v1.2.0 binary
EOF_HELP
    exit 0
}

MODE=""
VERSION=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
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
        bin|remote)
            MODE="bin"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

START_DIR=$(pwd)

if [ -f "$START_DIR/PKGBUILD" ] && [ -f "$START_DIR/Cargo.toml" ] && [ -z "$VERSION" ]; then
    echo "Detected local repository..."
    [ -z "$MODE" ] && MODE="local"

    if [ "$MODE" = "bin" ]; then
        TMP_DIR=$(mktemp -d)
        trap 'rm -rf "$TMP_DIR"' EXIT
        cp "$START_DIR/PKGBUILD.bin" "$TMP_DIR/PKGBUILD"
        cp "$START_DIR/keyboard-middleware.install" "$TMP_DIR/"
        cd "$TMP_DIR"
        makepkg -si
    else
        TMP_DIR=$(mktemp -d)
        trap 'rm -rf "$TMP_DIR"' EXIT
        echo "Copying source files to temporary directory..."
        cd "$START_DIR"
        
        # Get tracked files that exist, plus untracked but trackable files
        {
            git ls-files --cached --exclude-standard | while IFS= read -r file; do
                [ -e "$file" ] && echo "$file"
            done
            git ls-files --others --exclude-standard
        } | tar -czf - -T - | (cd "$TMP_DIR" && tar xzf -)
        
        cd "$TMP_DIR"
        
        # Check for unstaged changes and append +wip to version if needed
        # Check from original source directory, not temp directory
        cd "$START_DIR"
        if ! git diff --quiet || ! git diff --cached --quiet 2>/dev/null; then
            echo "Detected unstaged changes, appending +wip to version..."
            # Modify version in Cargo.toml to include +wip
            sed -i 's/^version = "\([^"]*\)"/version = "\1+wip"/' "$TMP_DIR/Cargo.toml"
            # Remove Cargo.lock to let Cargo regenerate it with new version
            rm -f "$TMP_DIR/Cargo.lock"
            # Remove --locked flag from PKGBUILD to allow Cargo to update lock file
            sed -i 's/cargo build --release --locked/cargo build --release/' "$TMP_DIR/PKGBUILD"
            # Append +wip to PKGBUILD version field
            sed -i 's/^pkgver=.*/&+wip/' "$TMP_DIR/PKGBUILD"
        fi
        cd "$TMP_DIR"
        
        echo "Building package as normal user..."
        makepkg

        echo "Installing package as root..."
        sudo -v
        sudo pacman -U --noconfirm *.pkg.tar.zst
    fi
else
    echo "Remote install..."
    [ -z "$MODE" ] && MODE="bin"
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT
    cd "$TMP_DIR"
        if [ "$MODE" = "bin" ]; then
            if [ -n "$VERSION" ]; then
                # Try exact match first, then find newest matching version
                if curl -fsSL "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/$VERSION/PKGBUILD.bin" >/dev/null 2>&1; then
                    curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/$VERSION/PKGBUILD.bin"
                    curl -fsSL -o keyboard-middleware.install "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/$VERSION/keyboard-middleware.install"
                else
                    echo "Finding newest version matching $VERSION..."
                    LATEST_TAG=$(git ls-remote --tags https://github.com/fibsussy/keyboard-middleware.git \
                        | grep "refs/tags/.*$VERSION" \
                        | sed 's|.*/\(.*\)|\1|' \
                        | sort -V \
                        | tail -n1)
                    if [ -n "$LATEST_TAG" ]; then
                        echo "Using version: $LATEST_TAG"
                        curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/$LATEST_TAG/PKGBUILD.bin"
                        curl -fsSL -o keyboard-middleware.install "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/$LATEST_TAG/keyboard-middleware.install"
                    else
                        echo "Error: No version found matching $VERSION"
                        exit 1
                    fi
                fi
            else
                curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/PKGBUILD.bin"
                curl -fsSL -o keyboard-middleware.install "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/keyboard-middleware.install"
            fi
    else
        if [ -n "$VERSION" ]; then
            # Try exact match first
            if git ls-remote --tags https://github.com/fibsussy/keyboard-middleware.git | grep -q "refs/tags/$VERSION$"; then
                git -c advice.detachedHead=false clone --branch "$VERSION" https://github.com/fibsussy/keyboard-middleware.git repo
                cd repo
            else
                # Find newest matching version
                echo "Finding newest version matching $VERSION..."
                LATEST_TAG=$(git ls-remote --tags https://github.com/fibsussy/keyboard-middleware.git \
                    | grep "refs/tags/.*$VERSION" \
                    | sed 's|.*/\(.*\)|\1|' \
                    | sort -V \
                    | tail -n1)
                if [ -n "$LATEST_TAG" ]; then
                    echo "Using version: $LATEST_TAG"
                    git -c advice.detachedHead=false clone --branch "$LATEST_TAG" https://github.com/fibsussy/keyboard-middleware.git repo
                    cd repo
                else
                    echo "Error: No version found matching $VERSION"
                    exit 1
                fi
            fi
        else
            git clone https://github.com/fibsussy/keyboard-middleware.git repo
            cd repo
        fi
        curl -fsSL -o PKGBUILD "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/PKGBUILD"
        curl -fsSL -o keyboard-middleware.install "https://raw.githubusercontent.com/fibsussy/keyboard-middleware/main/keyboard-middleware.install"
    fi

    echo "Building package as normal user..."
    makepkg

    echo "Installing package as root..."
    sudo -v
    sudo pacman -U --noconfirm *.pkg.tar.zst
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Installation complete! To enable the services:"
echo ""
echo "  Root daemon (required):"
echo "    sudo systemctl enable --now keyboard-middleware.service"
echo ""
echo "  Niri watcher (optional, for auto game mode):"
echo "    systemctl --user enable --now keyboard-middleware-niri.service"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
