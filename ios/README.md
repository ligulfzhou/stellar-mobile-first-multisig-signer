# iOS App (SwiftUI)

Native signer for reviewing pending proposals and approving/rejecting on-chain.

## Quick start

```bash
# From repo root — bindings, static libs, Xcode project
just ios-setup

open ios/VaultSigner.xcodeproj
```

Select an iOS Simulator and Run.

### Tabs

| Tab | Purpose |
|-----|---------|
| **Vault** | Dashboard, pending proposals, approve/reject |
| **Create** | Deploy new N-of-M vault via factory |
| **Settings** | Keychain key, vault/factory/RPC, demo vault |

The app ships with the demo testnet vault pre-filled. Import your signer secret (S...) in **Settings**, then use **Vault** to sign or **Create** to deploy a new treasury.

## What’s included

| Piece | Location |
|-------|----------|
| SwiftUI UI | `ios/VaultSigner/*.swift` |
| UniFFI Swift | `bindings/swift/vault_signer_ffi.swift` |
| Rust static lib (sim / device) | `ios/Vendor/{ios-sim,ios}/` |
| Xcode project spec | `ios/project.yml` |

## Rebuild Rust after API changes

```bash
just ffi          # regenerate Swift bindings
just ios-lib      # rebuild static libraries only
```

Then rebuild in Xcode.

## Production notes

- Secrets are stored in **Keychain** (`KeychainStore.swift`)
- Ship Rust as **XCFramework** for App Store (see `scripts/build-ios-lib.sh`)
- Hardened flow: `buildApproveTx` + Secure Enclave / passkey signing

## CLI parity

Same vault as `just e2e-testnet`:

```
CCJ3AAZCSG3MXY3WQ4BX6XZQBSV4T7QERHVP5LKKIQDHTXJE5JVBXP5Q
```
