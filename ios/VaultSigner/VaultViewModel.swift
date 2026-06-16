import Foundation

/// App state for vault dashboard and signing flows.
@MainActor
final class VaultViewModel: ObservableObject {
    @Published var vaultAddress: String = UserDefaults.standard.string(forKey: "vaultAddress")
        ?? AppConfig.defaultTestnetVault
    @Published var factoryAddress: String = UserDefaults.standard.string(forKey: "factoryAddress")
        ?? AppConfig.defaultTestnetFactory
    @Published var network: String = UserDefaults.standard.string(forKey: "network") ?? "testnet"
    @Published var rpcUrl: String = UserDefaults.standard.string(forKey: "rpcUrl")
        ?? AppConfig.defaultRpcUrl
    @Published var configName: String = ""
    @Published var threshold: UInt32 = 0
    @Published var signerCount: UInt32 = 0
    @Published var proposalCount: UInt64 = 0
    @Published var pending: [PendingRow] = []
    @Published var publicKey: String = ""
    @Published var statusMessage: String = ""
    @Published var isLoading = false
    @Published var lastTxHash: String = ""

    private let signer = VaultSigner()
    private var refreshTask: Task<Void, Never>?
    private var lastLoadedVault: String?
    private var lastRefreshTime: Date?
    private static let cacheTTL: TimeInterval = 45

    init() {
        if let secret = KeychainStore.loadSecret() {
            publicKey = (try? signer.publicKeyFromSecret(secret: secret)) ?? ""
        }
    }

    var rpcOptional: String? { rpcUrl.isEmpty ? nil : rpcUrl }

    func saveSettings() {
        UserDefaults.standard.set(vaultAddress, forKey: "vaultAddress")
        UserDefaults.standard.set(factoryAddress, forKey: "factoryAddress")
        UserDefaults.standard.set(network, forKey: "network")
        UserDefaults.standard.set(rpcUrl, forKey: "rpcUrl")
    }

    func useDemoVault() {
        vaultAddress = AppConfig.defaultTestnetVault
        factoryAddress = AppConfig.defaultTestnetFactory
        saveSettings()
        refresh(force: true)
    }

    func saveSecret(_ secret: String) {
        do {
            try KeychainStore.saveSecret(secret)
            publicKey = try signer.publicKeyFromSecret(secret: secret)
            statusMessage = "Signer ready"
        } catch {
            statusMessage = error.localizedDescription
        }
    }

    func clearSecret() {
        KeychainStore.deleteSecret()
        publicKey = ""
        statusMessage = "Secret removed from Keychain"
    }

    func refresh(force: Bool = false) {
        guard !vaultAddress.isEmpty else {
            statusMessage = "Select or create a vault"
            return
        }
        if isLoading { return }
        if !force,
           lastLoadedVault == vaultAddress,
           let lastRefreshTime,
           Date().timeIntervalSince(lastRefreshTime) < Self.cacheTTL {
            return
        }

        refreshTask?.cancel()
        isLoading = true
        if force { statusMessage = "" }

        let vault = vaultAddress
        let net = network
        let rpc = rpcOptional

        refreshTask = Task {
            let (snap, error) = await VaultLoader.loadSnapshot(vault: vault, network: net, rpcUrl: rpc)
            guard !Task.isCancelled else { return }
            applySnapshot(snap, error: error, vault: vault)
        }
    }

    func refreshIfNeeded() {
        refresh(force: false)
    }

    private func applySnapshot(_ snap: VaultSnapshot?, error: String?, vault: String) {
        isLoading = false
        if let snap {
            configName = snap.name
            threshold = snap.threshold
            signerCount = snap.signerCount
            proposalCount = snap.proposalCount
            pending = snap.pending
            statusMessage = snap.pending.isEmpty ? "All caught up" : "\(snap.pending.count) awaiting signature"
            lastLoadedVault = vault
            lastRefreshTime = Date()
        } else if let error {
            statusMessage = error
        }
    }

    func createVault(name: String, signers: [String], threshold: UInt32) {
        guard !name.isEmpty else {
            statusMessage = "Enter a vault name"
            return
        }
        guard !isLoading else { return }

        let secret: String
        do { secret = try secretOrError() } catch {
            statusMessage = error.localizedDescription
            return
        }

        isLoading = true
        statusMessage = "Deploying vault…"

        let factory = factoryAddress
        let net = network
        let rpc = rpcOptional
        let vaultName = sanitizeVaultName(name)

        Task {
            let (vaultId, error) = await VaultLoader.createVault(
                factory: factory,
                network: net,
                secret: secret,
                name: vaultName,
                signers: signers,
                threshold: threshold,
                rpcUrl: rpc
            )
            guard !Task.isCancelled else { return }
            isLoading = false
            if let vaultId {
                vaultAddress = vaultId
                saveSettings()
                lastTxHash = ""
                statusMessage = "Vault created"
                lastLoadedVault = nil
                refresh(force: true)
            } else if let error {
                statusMessage = error
            }
        }
    }

    func approve(proposalId: UInt64) { act(proposalId: proposalId, approve: true) }
    func reject(proposalId: UInt64) { act(proposalId: proposalId, approve: false) }

    private func act(proposalId: UInt64, approve: Bool) {
        guard !isLoading else { return }

        let secret: String
        do { secret = try secretOrError() } catch {
            statusMessage = error.localizedDescription
            return
        }

        isLoading = true
        statusMessage = approve ? "Approving…" : "Rejecting…"

        let vault = vaultAddress
        let net = network
        let rpc = rpcOptional

        Task {
            let (hash, error) = await VaultLoader.actOnProposal(
                approve: approve,
                vault: vault,
                network: net,
                secret: secret,
                proposalId: proposalId,
                rpcUrl: rpc
            )
            guard !Task.isCancelled else { return }
            isLoading = false
            if let hash {
                lastTxHash = hash
                statusMessage = approve ? "Approved" : "Rejected"
                lastLoadedVault = nil
                refresh(force: true)
            } else if let error {
                statusMessage = error
            }
        }
    }

    private func secretOrError() throws -> String {
        guard let secret = KeychainStore.loadSecret(), !secret.isEmpty else {
            throw SignerError.Generic(msg: "Import your signer key in Settings first")
        }
        return secret
    }

    private func sanitizeVaultName(_ raw: String) -> String {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        let allowed = trimmed.filter { $0.isLetter || $0.isNumber || $0 == "_" }
        let name = String(allowed.prefix(32))
        return name.isEmpty ? "vault" : name
    }
}
