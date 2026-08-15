#!/bin/bash
# Generate man pages and shell completions for luppo
# Run after: cargo build --release

set -e

BINARY="./target/release/luppo"
OUT_DIR="./man"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found at $BINARY. Run 'cargo build --release' first."
    exit 1
fi

mkdir -p "$OUT_DIR"

echo "Generating man page..."
$BINARY --help-man > "$OUT_DIR/luppo.1" 2>/dev/null || {
    echo "Man page generation via --help-man not supported, trying alternative..."
    # Alternative: use clap_mangen programmatically
    cargo run --features generate --bin gen_man 2>/dev/null || {
        echo "Could not generate man page automatically."
        echo "You can manually create man page from: $BINARY --help"
    }
}

echo "Generating shell completions..."
for shell in bash zsh fish powershell; do
    $BINARY --generate-completion $shell > "$OUT_DIR/luppo.$shell" 2>/dev/null || {
        echo "Completion generation for $shell not supported via CLI flag."
    }
done

echo "Done. Output in $OUT_DIR/"
ls -la "$OUT_DIR/"