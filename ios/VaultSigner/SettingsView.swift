import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var model: VaultViewModel
    @State private var secretInput = ""
    @State private var showSecretField = false

    var body: some View {
        NavigationStack {
            Form {
                Section("Active vault") {
                    TextField("Vault C…", text: $model.vaultAddress)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    TextField("Factory C…", text: $model.factoryAddress)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Picker("Network", selection: $model.network) {
                        Text("Testnet").tag("testnet")
                        Text("Mainnet").tag("mainnet")
                    }
                    TextField("RPC URL", text: $model.rpcUrl)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Button("Save") { model.saveSettings() }
                    Button("Use demo vault") { model.useDemoVault() }
                }

                Section("Your signer key") {
                    if !model.publicKey.isEmpty {
                        LabeledContent("Public key") {
                            Text(model.publicKey)
                                .font(.caption.monospaced())
                                .multilineTextAlignment(.trailing)
                        }
                    }
                    if showSecretField {
                        SecureField("Secret key (S…)", text: $secretInput)
                            .textInputAutocapitalization(.never)
                        Button("Save to Keychain") {
                            model.saveSecret(secretInput)
                            secretInput = ""
                            showSecretField = false
                        }
                        .disabled(secretInput.isEmpty)
                    } else {
                        Button(model.publicKey.isEmpty ? "Import secret key" : "Replace secret key") {
                            showSecretField = true
                        }
                        if !model.publicKey.isEmpty {
                            Button("Remove from Keychain", role: .destructive) {
                                model.clearSecret()
                            }
                        }
                    }
                    Text("Stored locally in Keychain. Never uploaded.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Section("About") {
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("Contract", value: "Soroban Vault")
                }
            }
            .navigationTitle("Settings")
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(VaultViewModel())
}
