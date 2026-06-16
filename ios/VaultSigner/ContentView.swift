import SwiftUI

struct ContentView: View {
    @EnvironmentObject var model: VaultViewModel
    @State private var secretKey = ""

    var body: some View {
        NavigationStack {
            List {
                Section("Vault") {
                    TextField("Contract address (C...)", text: $model.vaultAddress)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Picker("Network", selection: $model.network) {
                        Text("Testnet").tag("testnet")
                        Text("Mainnet").tag("mainnet")
                    }
                    Button("Save & Refresh") {
                        model.saveVaultAddress()
                        model.refresh()
                    }
                    .disabled(model.isLoading)
                }

                if !model.configName.isEmpty {
                    Section("Config") {
                        LabeledContent("Name", value: model.configName)
                        LabeledContent("Threshold", value: "\(model.threshold)")
                    }
                }

                Section("Pending proposals") {
                    if model.pending.isEmpty {
                        Text("None")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(model.pending) { row in
                            VStack(alignment: .leading, spacing: 4) {
                                Text("#\(row.id) · \(row.proposalType)")
                                    .font(.headline)
                                Text("Approvals \(row.approvals) · Rejections \(row.rejections)")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Button("Approve") {
                                    model.approve(proposalId: row.id, secret: secretKey)
                                }
                                .disabled(secretKey.isEmpty || model.isLoading)
                            }
                            .padding(.vertical, 4)
                        }
                    }
                }

                Section("Signer key") {
                    SecureField("Secret key (S...) — stays on device", text: $secretKey)
                        .textInputAutocapitalization(.never)
                    Text("Use Keychain in production; this demo keeps the secret in memory only.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if !model.statusMessage.isEmpty {
                    Section {
                        Text(model.statusMessage)
                            .font(.footnote)
                    }
                }
            }
            .navigationTitle("Vault Signer")
            .refreshable { model.refresh() }
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(VaultViewModel())
}
