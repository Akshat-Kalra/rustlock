#!/bin/bash
# Usage: ./install-native-host.sh <extension-id>
# Extension ID found in chrome://extensions after loading unpacked
set -e

EXTENSION_ID="${1:?Usage: $0 <extension-id>}"

# Build the host binary
cargo build --release --bin rustlock-host

HOST_BIN="$(pwd)/target/release/rustlock-host"

if [ ! -f "$HOST_BIN" ]; then
  echo "Error: rustlock-host binary not found at $HOST_BIN" >&2
  exit 1
fi

MANIFEST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
mkdir -p "$MANIFEST_DIR"

cat > "$MANIFEST_DIR/com.rustlock.host.json" << EOF
{
  "name": "com.rustlock.host",
  "description": "Rustlock password manager native host",
  "path": "$HOST_BIN",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$EXTENSION_ID/"]
}
EOF

echo "✓ Native host registered for extension $EXTENSION_ID"
echo "  Host binary: $HOST_BIN"
echo "  Manifest:    $MANIFEST_DIR/com.rustlock.host.json"
