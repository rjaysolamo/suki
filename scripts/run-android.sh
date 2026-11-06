#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

export ANDROID_HOME="${ANDROID_HOME:-/home/rickyler321/android-sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-/home/rickyler321/android-sdk/ndk/27.2.12479018}"
export NDK_HOME="${NDK_HOME:-/home/rickyler321/android-sdk/ndk/27.2.12479018}"
export PATH="/home/rickyler321/.local/bin:$ANDROID_HOME/platform-tools:$PATH"

echo "📱 Checking Android Emulator..."
DEVICE=$(adb devices 2>/dev/null | grep -E "emulator-[0-9]+" | head -n1 | awk '{print $1}' | tr -d '\r')

if [ -z "$DEVICE" ]; then
    AVD=$(emulator -list-avds 2>/dev/null | head -n1 | tr -d '\r')
    if [ -z "$AVD" ]; then
        echo "❌ No Android Virtual Device (AVD) found."
        echo "Please create an emulator in Android Studio Device Manager first."
        exit 1
    fi
    echo "🚀 Starting emulator with clean Cold Boot (no corrupted snapshot): $AVD..."
    emulator -avd "$AVD" -no-snapshot-load &
    EMU_PID=$!
    
    echo "⏳ Waiting for emulator to boot up..."
    until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
        sleep 2
    done
    echo "✅ Emulator is ready!"
fi

echo "📦 Running Suki Dioxus Mobile App on Android..."
if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "⚙️ Setting up Android NDK first..."
    "$DIR/scripts/setup-ndk.sh"
fi
cd "$DIR/mobile"
if command -v dx &> /dev/null; then
    dx serve --platform android
else
    echo "ℹ️ Dioxus CLI (dx) is not yet installed."
    echo "To install: cargo install dioxus-cli --version 0.7.10 --locked"
fi
