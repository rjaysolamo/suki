# Suki Mobile

See [Setup](../../docs/SETUP.md#native-mobile-builds) for the Android and iOS commands and how to set the API origin. Run builds with Dioxus CLI 0.7.10. Android requires API 28 or newer. The build script preserves 16 KB page alignment on Android.

The app keeps its access credential, basket, and favorites in memory. Android Keystore and iOS Keychain protect the remembered refresh credential. The web preview uses same-origin HttpOnly cookies. The server owns payments, order authorization, prices, and routing.


The optional browser preview reuses the Rust Dioxus screens for development. Use native device testing for platform behavior.


