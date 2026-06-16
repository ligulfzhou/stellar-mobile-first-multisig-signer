import SwiftUI

struct VaultDashboardView: View {
    @EnvironmentObject var model: VaultViewModel

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    heroCard
                    if !model.configName.isEmpty { statsRow }
                    pendingSection
                    if !model.statusMessage.isEmpty { statusBanner }
                }
                .padding()
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("Treasury")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        model.refresh(force: true)
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                    .disabled(model.isLoading)
                }
            }
            .refreshable { model.refresh(force: true) }
            .overlay {
                if model.isLoading && model.configName.isEmpty { ProgressView().scaleEffect(1.2) }
            }
            .task(id: model.vaultAddress) {
                model.refreshIfNeeded()
            }
        }
    }

    private var heroCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "lock.shield.fill")
                    .font(.title2)
                    .foregroundStyle(.white.opacity(0.9))
                Spacer()
                Text(model.network.uppercased())
                    .font(.caption.weight(.semibold))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 4)
                    .background(.white.opacity(0.2))
                    .clipShape(Capsule())
            }
            Text(model.configName.isEmpty ? "Multisig Vault" : model.configName)
                .font(.title2.bold())
                .foregroundStyle(.white)
            Text(truncatedAddress(model.vaultAddress))
                .font(.caption.monospaced())
                .foregroundStyle(.white.opacity(0.85))
            if model.threshold > 0 {
                Text("\(model.threshold)-of-\(model.signerCount) multisig")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.9))
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(AppTheme.heroGradient)
        .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private var statsRow: some View {
        HStack(spacing: 12) {
            statTile(title: "Pending", value: "\(model.pending.count)")
            statTile(title: "Proposals", value: "\(model.proposalCount)")
            statTile(title: "Signers", value: "\(model.signerCount)")
        }
    }

    private func statTile(title: String, value: String) -> some View {
        VStack(spacing: 4) {
            Text(value).font(.title2.bold())
            Text(title).font(.caption).foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .vaultCard()
    }

    private var pendingSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Awaiting your signature")
                .font(.headline)

            if model.publicKey.isEmpty {
                Label("Import your key in Settings to sign", systemImage: "key.fill")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .vaultCard()
            } else if model.pending.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "checkmark.seal.fill")
                        .font(.largeTitle)
                        .foregroundStyle(AppTheme.accent)
                    Text("No pending proposals")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 24)
                .vaultCard()
            } else {
                ForEach(model.pending) { row in
                    ProposalCard(row: row, threshold: model.threshold) {
                        model.approve(proposalId: row.id)
                    } onReject: {
                        model.reject(proposalId: row.id)
                    }
                    .disabled(model.isLoading || model.publicKey.isEmpty)
                }
            }
        }
    }

    private var statusBanner: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(model.statusMessage)
                .font(.footnote)
            if !model.lastTxHash.isEmpty {
                Text(model.lastTxHash)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(AppTheme.accent.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    private func truncatedAddress(_ address: String) -> String {
        guard address.count > 12 else { return address }
        return "\(address.prefix(6))…\(address.suffix(6))"
    }
}

struct ProposalCard: View {
    let row: PendingRow
    let threshold: UInt32
    let onApprove: () -> Void
    let onReject: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("#\(row.id)")
                    .font(.caption.weight(.bold))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(AppTheme.accent.opacity(0.15))
                    .clipShape(Capsule())
                Text(row.typeLabel)
                    .font(.headline)
                Spacer()
                Text(row.status)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            ProgressView(value: Double(row.approvals), total: Double(max(threshold, 1)))
                .tint(AppTheme.accent)

            Text("\(row.approvals) / \(threshold) approvals")
                .font(.caption)
                .foregroundStyle(.secondary)

            HStack {
                Button("Approve", action: onApprove)
                    .buttonStyle(.borderedProminent)
                    .tint(AppTheme.accent)
                Button("Reject", role: .destructive, action: onReject)
                    .buttonStyle(.bordered)
            }
        }
        .vaultCard()
    }
}

#Preview {
    VaultDashboardView()
        .environmentObject(VaultViewModel())
}
