#!/bin/bash
# 自动创建 Sidecar 平台特定符号链接

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$PROJECT_ROOT/target/release"
BINARY_NAME="server"

# 检测当前平台
case "$(uname -s)" in
  Darwin)
    case "$(uname -m)" in
      x86_64)  TRIPLE="x86_64-apple-darwin" ;;
      arm64)   TRIPLE="aarch64-apple-darwin" ;;
      *)       echo "Unsupported macOS architecture: $(uname -m)"; exit 1 ;;
    esac
    ;;
  Linux)
    TRIPLE="x86_64-unknown-linux-gnu"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    TRIPLE="x86_64-pc-windows-msvc.exe"
    ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

# 检查二进制是否存在
if [ ! -f "$TARGET_DIR/$BINARY_NAME" ]; then
  echo "❌ Error: Binary not found: $TARGET_DIR/$BINARY_NAME"
  echo "   Please run: cargo build --release --bin server"
  exit 1
fi

# 创建符号链接
cd "$TARGET_DIR"
if [ -L "$BINARY_NAME-$TRIPLE" ]; then
  # 符号链接已存在，删除重新创建
  rm "$BINARY_NAME-$TRIPLE"
fi

ln -sf "$BINARY_NAME" "$BINARY_NAME-$TRIPLE"
echo "✅ Created symlink: $BINARY_NAME-$TRIPLE -> $BINARY_NAME"
ls -lh "$BINARY_NAME-$TRIPLE"
