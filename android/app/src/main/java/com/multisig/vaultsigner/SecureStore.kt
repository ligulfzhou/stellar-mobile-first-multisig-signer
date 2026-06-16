package com.multisig.vaultsigner

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

class SecureStore(context: Context) {
    private val prefs = EncryptedSharedPreferences.create(
        context,
        "vault_signer_secrets",
        MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build(),
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    fun loadSecret(): String? = prefs.getString(KEY_SECRET, null)

    fun saveSecret(secret: String) {
        prefs.edit().putString(KEY_SECRET, secret).apply()
    }

    fun clearSecret() {
        prefs.edit().remove(KEY_SECRET).apply()
    }

    companion object {
        private const val KEY_SECRET = "signer-secret"
    }
}
