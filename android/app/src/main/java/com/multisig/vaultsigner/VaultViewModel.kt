package com.multisig.vaultsigner

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.vault_signer_ffi.FfiProposalSummary
import uniffi.vault_signer_ffi.SignerException
import uniffi.vault_signer_ffi.VaultSigner

data class VaultUiState(
    val vaultAddress: String = AppConfig.DEFAULT_TESTNET_VAULT,
    val factoryAddress: String = AppConfig.DEFAULT_TESTNET_FACTORY,
    val network: String = "testnet",
    val rpcUrl: String = AppConfig.DEFAULT_RPC_URL,
    val configName: String = "",
    val threshold: UInt = 0u,
    val signerCount: UInt = 0u,
    val proposalCount: ULong = 0u,
    val pending: List<FfiProposalSummary> = emptyList(),
    val publicKey: String = "",
    val statusMessage: String = "",
    val isLoading: Boolean = false,
    val lastTxHash: String = "",
)

class VaultViewModel(app: Application) : AndroidViewModel(app) {
    private val signer = VaultSigner()
    private val secureStore = SecureStore(app.applicationContext)
    private val prefs = app.getSharedPreferences("vault_signer", 0)

    private val _ui = MutableStateFlow(loadInitialState())
    val ui: StateFlow<VaultUiState> = _ui.asStateFlow()

    init {
        secureStore.loadSecret()?.let { secret ->
            runCatching { signer.publicKeyFromSecret(secret) }
                .onSuccess { pk -> _ui.update { it.copy(publicKey = pk) } }
        }
    }

    private fun loadInitialState(): VaultUiState {
        return VaultUiState(
            vaultAddress = prefs.getString("vaultAddress", AppConfig.DEFAULT_TESTNET_VAULT)!!,
            factoryAddress = prefs.getString("factoryAddress", AppConfig.DEFAULT_TESTNET_FACTORY)!!,
            network = prefs.getString("network", "testnet")!!,
            rpcUrl = prefs.getString("rpcUrl", AppConfig.DEFAULT_RPC_URL)!!,
        )
    }

    private fun rpc() = _ui.value.rpcUrl.ifBlank { null }

    fun updateVaultAddress(v: String) = _ui.update { it.copy(vaultAddress = v) }
    fun updateFactoryAddress(v: String) = _ui.update { it.copy(factoryAddress = v) }
    fun updateNetwork(v: String) = _ui.update { it.copy(network = v) }
    fun updateRpcUrl(v: String) = _ui.update { it.copy(rpcUrl = v) }

    fun saveSettings() {
        val s = _ui.value
        prefs.edit()
            .putString("vaultAddress", s.vaultAddress)
            .putString("factoryAddress", s.factoryAddress)
            .putString("network", s.network)
            .putString("rpcUrl", s.rpcUrl)
            .apply()
    }

    fun useDemoVault() {
        _ui.update {
            it.copy(
                vaultAddress = AppConfig.DEFAULT_TESTNET_VAULT,
                factoryAddress = AppConfig.DEFAULT_TESTNET_FACTORY,
            )
        }
        saveSettings()
        refresh()
    }

    fun saveSecret(secret: String) {
        secureStore.saveSecret(secret)
        runCatching { signer.publicKeyFromSecret(secret) }
            .onSuccess { pk -> _ui.update { it.copy(publicKey = pk, statusMessage = "Signer ready") } }
            .onFailure { e -> _ui.update { it.copy(statusMessage = e.message ?: "Invalid secret") } }
    }

    fun clearSecret() {
        secureStore.clearSecret()
        _ui.update { it.copy(publicKey = "", statusMessage = "Secret removed") }
    }

    fun refresh() {
        val state = _ui.value
        if (state.vaultAddress.isBlank()) {
            _ui.update { it.copy(statusMessage = "Select or create a vault") }
            return
        }
        viewModelScope.launch {
            _ui.update { it.copy(isLoading = true, statusMessage = "") }
            try {
                val cfg = withContext(Dispatchers.IO) {
                    signer.getVaultConfig(state.vaultAddress, state.network, rpc())
                }
                val pending = withContext(Dispatchers.IO) {
                    signer.listPendingProposals(state.vaultAddress, state.network, rpc())
                }
                _ui.update {
                    it.copy(
                        configName = cfg.name,
                        threshold = cfg.threshold,
                        signerCount = cfg.signerCount,
                        proposalCount = cfg.proposalCount,
                        pending = pending,
                        statusMessage = if (pending.isEmpty()) "All caught up" else "${pending.size} awaiting signature",
                        isLoading = false,
                    )
                }
            } catch (e: SignerException.Generic) {
                _ui.update { it.copy(statusMessage = e.msg, isLoading = false) }
            } catch (e: Exception) {
                _ui.update { it.copy(statusMessage = e.message ?: "Error", isLoading = false) }
            }
        }
    }

    fun createVault(name: String, signers: List<String>, threshold: UInt) {
        if (name.isBlank()) {
            _ui.update { it.copy(statusMessage = "Enter a vault name") }
            return
        }
        val secret = secureStore.loadSecret()
        if (secret.isNullOrBlank()) {
            _ui.update { it.copy(statusMessage = "Import your signer key in Settings first") }
            return
        }
        val state = _ui.value
        viewModelScope.launch {
            _ui.update { it.copy(isLoading = true, statusMessage = "Deploying vault…") }
            try {
                val vaultId = withContext(Dispatchers.IO) {
                    signer.createVault(
                        state.factoryAddress,
                        state.network,
                        secret,
                        sanitizeName(name),
                        signers,
                        threshold,
                        rpc(),
                    )
                }
                _ui.update { it.copy(vaultAddress = vaultId, statusMessage = "Vault created", isLoading = false) }
                saveSettings()
                refresh()
            } catch (e: SignerException.Generic) {
                _ui.update { it.copy(statusMessage = e.msg, isLoading = false) }
            } catch (e: Exception) {
                _ui.update { it.copy(statusMessage = e.message ?: "Error", isLoading = false) }
            }
        }
    }

    fun approve(id: ULong) = act(id, approve = true)
    fun reject(id: ULong) = act(id, approve = false)

    private fun act(proposalId: ULong, approve: Boolean) {
        val secret = secureStore.loadSecret()
        if (secret.isNullOrBlank()) {
            _ui.update { it.copy(statusMessage = "Import your signer key first") }
            return
        }
        val state = _ui.value
        viewModelScope.launch {
            _ui.update { it.copy(isLoading = true) }
            try {
                val hash = withContext(Dispatchers.IO) {
                    if (approve) {
                        signer.approveProposal(state.vaultAddress, state.network, secret, proposalId, rpc())
                    } else {
                        signer.rejectProposal(state.vaultAddress, state.network, secret, proposalId, rpc())
                    }
                }
                _ui.update {
                    it.copy(
                        lastTxHash = hash,
                        statusMessage = if (approve) "Approved" else "Rejected",
                        isLoading = false,
                    )
                }
                refresh()
            } catch (e: SignerException.Generic) {
                _ui.update { it.copy(statusMessage = e.msg, isLoading = false) }
            } catch (e: Exception) {
                _ui.update { it.copy(statusMessage = e.message ?: "Error", isLoading = false) }
            }
        }
    }

    override fun onCleared() {
        signer.close()
        super.onCleared()
    }

    private fun sanitizeName(raw: String): String {
        val cleaned = raw.trim().lowercase().filter { it.isLetterOrDigit() || it == '_' }.take(32)
        return cleaned.ifEmpty { "vault" }
    }
}
