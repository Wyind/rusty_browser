#!/bin/bash

# --- Configuration ---
APP_NAME="rusty_browser"
DEST_BIN="/usr/local/bin"
DEST_DESKTOP="/usr/share/applications"
ICON_NAME="applications-internet" # We use a standard system icon

# --- Checks ---
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust (cargo) is not installed. Please install Rust first."
    exit 1
fi

if ! command -v pkg-config &> /dev/null; then
    echo "WARNING: pkg-config not found. Compilation may fail if dev libraries are missing."
fi

# --- Main Script ---

echo "--- Building Rusty Browser (Release Mode) ---"
# Compile the Rust project
cargo build --release

if [ $? -ne 0 ]; then
    echo "ERROR: Compilation failed. Aborting installation."
    exit 1
fi

echo "--- Installing Files (Requires root privileges) ---"

# 1. Install Binary
echo "Installing binary to $DEST_BIN..."
# Use install command for proper permissions
sudo install -Dm755 "target/release/$APP_NAME" "$DEST_BIN/$APP_NAME"
if [ $? -ne 0 ]; then
    echo "FATAL: Failed to copy binary. Check permissions."
    exit 1
fi

# 2. Install Desktop Entry
echo "Installing .desktop file to $DEST_DESKTOP..."

# We create the .desktop file on the fly (since it's small)
sudo tee "$DEST_DESKTOP/$APP_NAME.desktop" > /dev/null << EOF
[Desktop Entry]
Name=Rusty Browser
GenericName=Web Browser
Comment=A privacy-focused, high-performance web browser built with Rust and WebKit.
Exec=$DEST_BIN/$APP_NAME %U
Icon=$ICON_NAME
Type=Application
Categories=Network;WebBrowser;
StartupNotify=true
MimeType=text/html;text/xml;application/xhtml+xml;application/vnd.mozilla.xul+xml;application/x-mimearchive;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/ftp;
EOF

# 3. Update Icon/MIME Cache
echo "Updating desktop database..."
sudo update-desktop-database "$DEST_DESKTOP"

echo "=========================================="
echo "INSTALLATION COMPLETE"
echo "You can now find 'Rusty Browser' in your application menu."
echo "=========================================="
