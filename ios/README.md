# iOS App (SwiftUI)

Minimal signer UI for pending proposal review and approve.

## Setup

1. Generate bindings from repo root:

   ```bash
   just ffi
   ```

2. Create an Xcode iOS App project (or open these sources in a new target).

3. Add to the target:
   - `ios/VaultSigner/*.swift`
   - `bindings/swift/vault_signer_ffi.swift`
   - `bindings/swift/vault_signer_ffiFFI.h` (Bridging Header)
   - Link `target/debug/libvault_signer_ffi.dylib` (device builds need `cargo build --release` for arm64 + XCFramework — see below)

4. Build Settings:
   - **Header Search Paths**: `$(PROJECT_DIR)/../../bindings/swift`
   - **Library Search Paths**: `$(PROJECT_DIR)/../../target/debug`
   - **Other Linker Flags**: `-lvault_signer_ffi`

5. Run on simulator, enter vault `C...` address, pull to refresh pending proposals.

## Production notes

- Store secrets in **Keychain**, never UserDefaults
- Ship Rust as **XCFramework** built for `aarch64-apple-ios` and simulator
- Use `build_approve_tx` + Secure Enclave signing for hardened flows

## XCFramework (later)

```bash
# Example — adjust targets for your toolchain
cargo build -p vault-signer-ffi --release --target aarch64-apple-ios
cargo build -p vault-signer-ffi --release --target aarch64-apple-ios-sim
# xcodebuild -create-xcframework ...
```
