import Foundation

struct PendingRow: Identifiable, Sendable {
    let id: UInt64
    let proposalType: String
    let approvals: UInt32
    let rejections: UInt32
    let status: String

    var typeLabel: String {
        proposalType.replacingOccurrences(of: "_", with: " ").capitalized
    }
}

struct VaultSnapshot: Sendable {
    let name: String
    let threshold: UInt32
    let signerCount: UInt32
    let proposalCount: UInt64
    let pending: [PendingRow]
}

/// Runs UniFFI RPC off the main thread.
enum VaultLoader {
    static func loadSnapshot(
        vault: String,
        network: String,
        rpcUrl: String?
    ) async -> (VaultSnapshot?, String?) {
        await Task.detached(priority: .userInitiated) {
            let ffi = VaultSigner()
            do {
                let cfg = try ffi.getVaultConfig(vault: vault, network: network, rpcUrl: rpcUrl)
                let list = try ffi.listPendingProposals(vault: vault, network: network, rpcUrl: rpcUrl)
                let pending = list.map {
                    PendingRow(
                        id: $0.id,
                        proposalType: $0.proposalType,
                        approvals: $0.approvalCount,
                        rejections: $0.rejectionCount,
                        status: $0.status
                    )
                }
                return (
                    VaultSnapshot(
                        name: cfg.name,
                        threshold: cfg.threshold,
                        signerCount: cfg.signerCount,
                        proposalCount: cfg.proposalCount,
                        pending: pending
                    ),
                    nil
                )
            } catch let err as SignerError {
                return (nil, err.localizedDescription)
            } catch {
                return (nil, error.localizedDescription)
            }
        }.value
    }

    static func createVault(
        factory: String,
        network: String,
        secret: String,
        name: String,
        signers: [String],
        threshold: UInt32,
        rpcUrl: String?
    ) async -> (String?, String?) {
        await Task.detached(priority: .userInitiated) {
            let ffi = VaultSigner()
            do {
                let id = try ffi.createVault(
                    factory: factory,
                    network: network,
                    secret: secret,
                    name: name,
                    signers: signers,
                    threshold: threshold,
                    rpcUrl: rpcUrl
                )
                return (id, nil)
            } catch let err as SignerError {
                return (nil, err.localizedDescription)
            } catch {
                return (nil, error.localizedDescription)
            }
        }.value
    }

    static func actOnProposal(
        approve: Bool,
        vault: String,
        network: String,
        secret: String,
        proposalId: UInt64,
        rpcUrl: String?
    ) async -> (String?, String?) {
        await Task.detached(priority: .userInitiated) {
            let ffi = VaultSigner()
            do {
                let hash: String
                if approve {
                    hash = try ffi.approveProposal(
                        vault: vault, network: network, secret: secret,
                        proposalId: proposalId, rpcUrl: rpcUrl
                    )
                } else {
                    hash = try ffi.rejectProposal(
                        vault: vault, network: network, secret: secret,
                        proposalId: proposalId, rpcUrl: rpcUrl
                    )
                }
                return (hash, nil)
            } catch let err as SignerError {
                return (nil, err.localizedDescription)
            } catch {
                return (nil, error.localizedDescription)
            }
        }.value
    }
}
