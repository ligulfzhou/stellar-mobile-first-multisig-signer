import SwiftUI

struct CreateVaultView: View {
    @EnvironmentObject var model: VaultViewModel
    @State private var vaultName = ""
    @State private var threshold: Int = 2
    @State private var extraSigners: [String] = [""]
    @State private var includeSelf = true

    private var allSigners: [String] {
        var list: [String] = []
        if includeSelf, !model.publicKey.isEmpty {
            list.append(model.publicKey)
        }
        list.append(contentsOf: extraSigners.map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty })
        return Array(Set(list))
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Text("Deploy a new on-chain multisig treasury. You must be one of the signers and pay the Soroban transaction fee.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                Section("Vault") {
                    TextField("Name (e.g. team_treasury)", text: $vaultName)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Stepper("Threshold: \(threshold)-of-\(max(allSigners.count, 1))", value: $threshold, in: 1...max(allSigners.count, 1))
                }

                Section("Signers") {
                    if model.publicKey.isEmpty {
                        Label("Import your key in Settings first", systemImage: "exclamationmark.triangle")
                            .foregroundStyle(.orange)
                    } else {
                        Toggle("Include me (\(truncated(model.publicKey)))", isOn: $includeSelf)
                    }

                    ForEach(extraSigners.indices, id: \.self) { index in
                        HStack {
                            TextField("Co-signer G… address", text: $extraSigners[index])
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                            if extraSigners.count > 1 {
                                Button(role: .destructive) {
                                    extraSigners.remove(at: index)
                                } label: {
                                    Image(systemName: "minus.circle.fill")
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }

                    if allSigners.count < AppConfig.maxSigners {
                        Button {
                            extraSigners.append("")
                        } label: {
                            Label("Add co-signer", systemImage: "person.badge.plus")
                        }
                    }

                    Text("\(allSigners.count) signer(s) · requires \(threshold) approval(s)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Section {
                    Button {
                        model.createVault(
                            name: vaultName,
                            signers: allSigners,
                            threshold: UInt32(threshold)
                        )
                    } label: {
                        HStack {
                            Spacer()
                            if model.isLoading {
                                ProgressView()
                            } else {
                                Text("Create vault on-chain")
                                    .fontWeight(.semibold)
                            }
                            Spacer()
                        }
                    }
                    .disabled(
                        model.isLoading ||
                        vaultName.isEmpty ||
                        allSigners.count < threshold ||
                        model.publicKey.isEmpty ||
                        !includeSelf
                    )
                }

                if !model.statusMessage.isEmpty {
                    Section {
                        Text(model.statusMessage)
                            .font(.footnote)
                        if !model.vaultAddress.isEmpty, model.statusMessage.contains("created") {
                            Text(model.vaultAddress)
                                .font(.caption.monospaced())
                        }
                    }
                }
            }
            .navigationTitle("Create Vault")
            .onChange(of: allSigners.count) { _, count in
                if threshold > count { threshold = max(count, 1) }
            }
        }
    }

    private func truncated(_ key: String) -> String {
        guard key.count > 10 else { return key }
        return "\(key.prefix(4))…\(key.suffix(4))"
    }
}

#Preview {
    CreateVaultView()
        .environmentObject(VaultViewModel())
}
