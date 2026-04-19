#!/bin/bash
set -e

# ShiftTab Installation Script
# This downloads the pre-compiled binary, verifies it, downloads the Zsh wrapper, and installs it.

VERSION="v0.1.0"
REPO="username/ShiftTab"
BASE_URL="https://github.com/$REPO/releases/download/$VERSION"

# (Note: These URLs will need to match your actual GitHub Releases!)
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    BIN_NAME="ShiftTab-linux-x86_64"
else
    echo "Error: Unsupported architecture $ARCH"
    exit 1
fi

BIN_URL="$BASE_URL/$BIN_NAME"
SHA_URL="$BASE_URL/$BIN_NAME.sha256"

INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/shifttab"
ZSH_PLUGIN_URL="https://raw.githubusercontent.com/$REPO/main/shifttab.zsh"

echo "=> Preparing directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"

echo "=> Downloading ShiftTab binary..."
curl -fsSL "$BIN_URL" -o "$INSTALL_DIR/ShiftTab"
curl -fsSL "$SHA_URL" -o "$INSTALL_DIR/ShiftTab.sha256"

echo "=> Verifying SHA256 Checksum..."
cd "$INSTALL_DIR"
if sha256sum -c "ShiftTab.sha256" 2>/dev/null; then
    echo "✓ Checksum OK"
else
    echo "✗ Checksum Failed! Aborting installation."
    rm -f "ShiftTab" "ShiftTab.sha256"
    exit 1
fi
rm -f "ShiftTab.sha256"

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

# Make sure .local/bin is in PATH for future sessions
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "⚠️  WARNING: $HOME/.local/bin is not in your PATH."
    echo "Please add 'export PATH=\"\$HOME/.local/bin:\$PATH\"' to your ~/.zshrc"
fi

echo ""
echo "=> 🎉 Installation Complete! Please restart your terminal or run:"
echo "   source ~/.zshrc"
