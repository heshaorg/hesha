#!/bin/bash
# Install script for Hesha CLI

set -e

echo "🚀 Installing Hesha..."

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Build release if not already built
if [ ! -f "$PROJECT_ROOT/target/release/hesha" ]; then
    echo "📦 Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin hesha
fi

# Get the full path to the binary
HESHA_PATH="$PROJECT_ROOT/target/release/hesha"

# Detect shell
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
    SHELL_NAME="zsh"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
    SHELL_NAME="bash"
else
    SHELL_RC="$HOME/.profile"
    SHELL_NAME="sh"
fi

# Create alias
ALIAS_LINE="alias hesha='$HESHA_PATH'"

# Check if alias already exists
if grep -q "alias hesha=" "$SHELL_RC" 2>/dev/null; then
    echo "⚠️  Updating existing hesha alias..."
    # Remove old alias
    sed -i.bak '/alias hesha=/d' "$SHELL_RC"
fi

# Add new alias
echo "" >> "$SHELL_RC"
echo "# Hesha CLI" >> "$SHELL_RC"
echo "$ALIAS_LINE" >> "$SHELL_RC"

echo "✅ Hesha installed successfully!"
echo ""
echo "📍 Binary location: $HESHA_PATH"
echo "📝 Alias added to: $SHELL_RC"
echo ""
echo "🔄 To use hesha immediately, run:"
echo "   source $SHELL_RC"
echo ""
echo "Or start a new terminal session."
echo ""
echo "📚 Usage examples:"
echo "   hesha setup    # Setup a new issuer"
echo "   hesha start    # Start the issuer node"
echo "   hesha stop     # Stop the issuer node"
echo "   hesha --help   # Show all commands"