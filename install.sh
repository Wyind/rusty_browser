#!/bin/bash

# --- Configuration ---
APP_NAME="rusty_browser"
DEST_BIN="/usr/local/bin"
DEST_DESKTOP="/usr/share/applications"
DEST_ICON_DIR="/usr/share/icons/hicolor/128x128/apps"

# --- CHANGE: Icon file path now points to src/logo.png ---
ICON_FILE="src/logo.png"
ICON_NAME="rusty-browser"
# --------------------------------------------------------

# --- Checks ---
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust (cargo) is not installed. Please install Rust first."
    exit 1
fi

if [ ! -f "$ICON_FILE" ]; then
    echo "WARNING: $ICON_FILE not found in source directory. Using generic icon."
    ICON_NAME="applications-internet"
else
    echo "Custom logo found at $ICON_FILE. Will install custom icon."
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
sudo install -Dm755 "target/release/$APP_NAME" "$DEST_BIN/$APP_NAME"

# 2. Install Custom Icon (Only if found)
if [ "$ICON_NAME" == "rusty-browser" ]; then
    echo "Installing custom icon to $DEST_ICON_DIR..."
    # Use the updated $ICON_FILE variable for the source path
    sudo install -Dm644 "$ICON_FILE" "$DEST_ICON_DIR/$ICON_NAME.png"
fi

# 3. Install Desktop Entry
echo "Installing .desktop file to $DEST_DESKTOP..."

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

# 4. Update Icon/MIME Cache
echo "Updating icon and desktop database..."
# Refresh the icon cache
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

# Refresh the application desktop database
sudo update-desktop-database "$DEST_DESKTOP"

echo "=========================================="
echo "INSTALLATION COMPLETE"
echo "Your custom logo should now appear for 'Rusty Browser'."
echo "=========================================="
