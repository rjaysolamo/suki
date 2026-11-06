#!/usr/bin/env bash
set -e

NDK_DIR="/home/rickyler321/android-sdk/ndk/27.2.12479018"
TEMP_ZIP="/home/rickyler321/android-sdk/.temp/android-ndk-r27c-linux.zip"

if [ -f "$NDK_DIR/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android28-clang" ]; then
    echo "✅ Android NDK 27.2 is already installed at $NDK_DIR"
    exit 0
fi

mkdir -p "/home/rickyler321/android-sdk/ndk" "/home/rickyler321/android-sdk/.temp"

echo "📥 Resuming Android NDK r27c download (total ~633MB)..."
curl -L -C - -o "$TEMP_ZIP" "https://dl.google.com/android/repository/android-ndk-r27c-linux.zip"

echo "📦 Extracting NDK to $NDK_DIR..."
unzip -q -o "$TEMP_ZIP" -d "/home/rickyler321/android-sdk/ndk/"

if [ -d "/home/rickyler321/android-sdk/ndk/android-ndk-r27c" ]; then
    rm -rf "$NDK_DIR"
    mv "/home/rickyler321/android-sdk/ndk/android-ndk-r27c" "$NDK_DIR"
fi

rm -f "$TEMP_ZIP"
echo "🎉 Android NDK installed successfully at $NDK_DIR!"

