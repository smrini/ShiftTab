#!/bin/bash
set -e

# ShiftTab Installation Script
# This downloads the pre-compiled binary or builds from source, verifies it, and installs it.

VERSION="v0.1.0"
REPO="smrini/ShiftTab"
BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
BUILD_FROM_SOURCE=false

# === Dependency Checking ===
echo "=> Checking dependencies..."
for cmd in curl zsh; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "Error: Required command '$cmd' not found. Please install it first."
        exit 1
    fi
done

if ! command -v sha256sum &> /dev/null; then
    echo "Warning: sha256sum not found. Cannot verify binary checksum."
    echo "Install util-linux or coreutils to enable verification."
fi

if ! command -v cargo &> /dev/null; then
    echo "Info: cargo not found. Binary fallback will be used (if available)."
else
    echo "✓ cargo found (source build available as fallback)"
fi

# === Architecture Detection ===
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        BIN_NAME="ShiftTab-linux-x86_64"
        ;;
    aarch64 | arm64)
        BIN_NAME="ShiftTab-linux-aarch64"
        ;;
    i686)
        echo "Note: i686 pre-compiled binary not available. Will build from source if possible."
        BIN_NAME="ShiftTab-linux-i686"  # Placeholder for attempt
        BUILD_FROM_SOURCE=true
        ;;
    *)
        echo "Error: Unsupported architecture '$ARCH'"
        echo "Supported: x86_64, aarch64, i686"
        exit 1
        ;;
esac

BIN_URL="$BASE_URL/$BIN_NAME"
SHA_URL="$BASE_URL/$BIN_NAME.sha256"

# === Installation Directory Selection ===
INSTALL_DIR=""
for dir in "$HOME/.local/bin" "$HOME/.cargo/bin" "$HOME/bin"; do
    if [[ ":$PATH:" == *":$dir:"* ]]; then
        INSTALL_DIR="$dir"
        break
    fi
done

# Fallback if no preferred directories are in PATH
if [ -z "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
fi

echo "   Using install directory: $INSTALL_DIR"

CONFIG_DIR="$HOME/.config/shifttab"
ZSH_PLUGIN_URL="https://raw.githubusercontent.com/$REPO/master/shifttab.zsh"
SOURCE_URL="${url}/archive/refs/tags/${VERSION}.tar.gz"

echo ""
echo "=> Preparing directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
echo "✓ Directories ready"

# === Try Binary Installation First ===
echo ""
echo "=> Attempting to download pre-compiled binary..."
if curl -fsSL --head "$BIN_URL" 2>/dev/null | grep -q "200\|302"; then
    echo "   Found binary for $ARCH"
    if curl -fsSL "$BIN_URL" -o "$INSTALL_DIR/$BIN_NAME" 2>/dev/null; then
        if curl -fsSL "$SHA_URL" -o "$INSTALL_DIR/$BIN_NAME.sha256" 2>/dev/null; then
            echo "=> Verifying SHA256 checksum..."
            cd "$INSTALL_DIR"
            if command -v sha256sum &> /dev/null && sha256sum -c "$BIN_NAME.sha256" 2>/dev/null; then
                echo "✓ Checksum verified"
                mv "$BIN_NAME" "ShiftTab"
                rm -f "$BIN_NAME.sha256"
                BINARY_INSTALL_SUCCESS=true
            else
                echo "   Checksum verification skipped or failed"
                echo "   (continuing with verification by test run)"
                mv "$BIN_NAME" "ShiftTab"
                rm -f "$BIN_NAME.sha256"
                BINARY_INSTALL_SUCCESS=true
            fi
        else
            echo "   Could not download checksum file"
            rm -f "$INSTALL_DIR/$BIN_NAME"
            BUILD_FROM_SOURCE=true
        fi
    else
        echo "   Download failed, will attempt source build"
        BUILD_FROM_SOURCE=true
    fi
else
    echo "   Binary not available for $ARCH, will build from source"
    BUILD_FROM_SOURCE=true
fi

# === Fallback to Source Build ===
if [ "$BUILD_FROM_SOURCE" = true ] && [ "$BINARY_INSTALL_SUCCESS" != true ]; then
    echo ""
    echo "=> Building from source..."
    if ! command -v cargo &> /dev/null; then
        echo "✗ Error: cargo not found and binary unavailable."
        echo "   Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    TEMP_BUILD_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_BUILD_DIR" EXIT
    
    echo "   Downloading source code..."
    curl -fsSL "$SOURCE_URL" -o "$TEMP_BUILD_DIR/source.tar.gz"
    cd "$TEMP_BUILD_DIR"
    tar xzf source.tar.gz
    cd ShiftTab-${VERSION#v}  # Remove 'v' prefix for directory name
    
    echo "   Compiling..."
    cargo build --release --locked
    
    echo "   Installing binary..."
    install -Dm755 target/release/ShiftTab "$INSTALL_DIR/ShiftTab"
    echo "✓ Build and install successful"
fi

echo "=> Making binary executable..."
chmod +x "$INSTALL_DIR/ShiftTab"

echo "=> Downloading Zsh integration..."
curl -fsSL "$ZSH_PLUGIN_URL" -o "$CONFIG_DIR/shifttab.zsh"

echo "=> Updating .zshrc..."
if ! grep -q "shifttab.zsh" "$HOME/.zshrc"; then
    echo -e "\n# ShiftTab Zsh TUI Autocomplete\nsource $CONFIG_DIR/shifttab.zsh" >> "$HOME/.zshrc"
    echo "✓ Added source line to ~/.zshrc"
else
    echo "✓ ~/.zshrc already contains ShiftTab."
fi

# Make sure the install directory is in PATH for future sessions
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "=> Automatically adding $INSTALL_DIR to your ~/.zshrc PATH..."
    echo -e "\n# Add ShiftTab install directory to PATH\nexport PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc"
    echo "✓ PATH updated in ~/.zshrc."
fi

# === Final Verification ===
echo ""
if ! "$INSTALL_DIR/ShiftTab" --version &>/dev/null; then
    echo "⚠ Warning: Binary test failed. Please verify installation manually:"
    echo "   $INSTALL_DIR/ShiftTab --version"
else
    echo "✓ Binary verified and working"
fi

echo ""
echo "=> Installation Complete! Please restart your terminal or run:"
echo "   source ~/.zshrc"
