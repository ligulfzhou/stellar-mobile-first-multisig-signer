import Foundation

/// Wraps UniFFI-generated VaultSigner (from `just ffi`).
/// Link `bindings/swift/vault_signer_ffi.swift` + `libvault_signer_ffi.dylib` in Xcode.
@MainActor
final class VaultViewModel: ObservableObject {
    @Published var vaultAddress: String = UserDefaults.standard.string(forKey: "vaultAddress") ?? ""
    @Published var network: String = "testnet"
    @Published var configName: String = ""
    @Published var threshold: UInt32 = 0
    @Published var pending: [PendingRow] = []
    @Published var statusMessage: String = ""
    @Published var isLoading = false

    private let signer = VaultSigner()

    struct PendingRow: Identifiable {
        let id: UInt64
        let proposalType: String
        let approvals: UInt32
        let rejections: UInt32
    }

    func saveVaultAddress() {
        UserDefaults.standard.set(vaultAddress, forKey: "vaultAddress")
    }

    func refresh() {
        guard !vaultAddress.isEmpty else {
            statusMessage = "Enter a vault contract address (C...)"
            return
        }
        isLoading = true
        statusMessage = ""
        defer { isLoading = false }

        do {
            let cfg = try signer.getVaultConfig(
                vault: vaultAddress,
                network: network,
                rpcUrl: nil
            )
            configName = cfg.name
            threshold = cfg.threshold

            let list = try signer.listPendingProposals(
                vault: vaultAddress,
                network: network,
                rpcUrl: nil
            )
            pending = list.map {
                PendingRow(
                    id: $0.id,
                    proposalType: $0.proposalType,
                    approvals: $0.approvalCount,
                    rejections: $0.rejectionCount
                )
            }
            statusMessage = pending.isEmpty ? "No pending proposals" : "\(pending.count) pending"
        } catch let err as SignerError {
            statusMessage = err.localizedDescription
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func approve(proposalId: UInt64, secret: String) {
        isLoading = true
        defer { isLoading = false }
        do {
            let hash = try signer.approveProposal(
                vault: vaultAddress,
                network: network,
                secret: secret,
                proposalId: proposalId,
                rpcUrl: nil
            )
            statusMessage = "Approved: \(hash)"
            refresh()
        } catch let err as SignerError {
            statusMessage = err.localizedDescription
        } catch {
            statusMessage = error.localizedDescription
        }
    }
}
