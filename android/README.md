# Android App (Jetpack Compose)

Native signer mirroring the iOS app: pending proposals, approve/reject.

## Quick start

```bash
just android-setup    # UniFFI bindings + libvault_signer_ffi.so for arm64
```

Open `android/` in **Android Studio** (Ladybug or newer), sync Gradle, run on an arm64 device or emulator.

## Layout

| Piece | Location |
|-------|----------|
| Compose UI | `android/app/src/main/java/com/multisig/vaultsigner/` |
| UniFFI Kotlin | `bindings/kotlin/uniffi/vault_signer_ffi/` |
| Native `.so` | `android/app/src/main/jniLibs/arm64-v8a/` |

## Rebuild Rust

```bash
just ffi
just android-lib
```

## Notes

- Secrets stored with **EncryptedSharedPreferences** (`SecureStore.kt`)
- Requires **NDK** + `rustup target add aarch64-linux-android`
- Demo vault: `CCJ3AAZCSG3MXY3WQ4BX6XZQBSV4T7QERHVP5LKKIQDHTXJE5JVBXP5Q`
