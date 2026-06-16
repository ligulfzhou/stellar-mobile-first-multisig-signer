package com.multisig.vaultsigner.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import com.multisig.vaultsigner.AppConfig
import com.multisig.vaultsigner.VaultViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(viewModel: VaultViewModel) {
    var tab by remember { mutableIntStateOf(0) }
    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        when (tab) {
                            0 -> "Treasury"
                            1 -> "Create Vault"
                            else -> "Settings"
                        },
                    )
                },
            )
        },
        bottomBar = {
            NavigationBar {
                NavigationBarItem(selected = tab == 0, onClick = { tab = 0 }, icon = { Icon(Icons.Default.Home, null) }, label = { Text("Vault") })
                NavigationBarItem(selected = tab == 1, onClick = { tab = 1 }, icon = { Icon(Icons.Default.Add, null) }, label = { Text("Create") })
                NavigationBarItem(selected = tab == 2, onClick = { tab = 2 }, icon = { Icon(Icons.Default.Settings, null) }, label = { Text("Settings") })
            }
        },
    ) { padding ->
        when (tab) {
            0 -> VaultTab(viewModel, Modifier.padding(padding))
            1 -> CreateVaultTab(viewModel, Modifier.padding(padding))
            else -> SettingsTab(viewModel, Modifier.padding(padding))
        }
    }
}

@Composable
private fun VaultTab(viewModel: VaultViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.ui.collectAsState()
    LazyColumn(modifier = modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        item {
            Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)) {
                Column(Modifier.padding(16.dp)) {
                    Text(state.configName.ifEmpty { "Multisig Vault" }, style = MaterialTheme.typography.titleLarge)
                    Text(state.vaultAddress, style = MaterialTheme.typography.bodySmall)
                    if (state.threshold > 0u) {
                        Text("${state.threshold}-of-${state.signerCount} multisig")
                    }
                    Spacer(Modifier.height(8.dp))
                    Button(onClick = viewModel::refresh, enabled = !state.isLoading) { Text("Refresh") }
                }
            }
        }
        if (state.pending.isEmpty()) {
            item { Text("No pending proposals", color = MaterialTheme.colorScheme.onSurfaceVariant) }
        } else {
            items(state.pending, key = { it.id }) { row ->
                Card {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("#${row.id} · ${row.proposalType}", style = MaterialTheme.typography.titleMedium)
                        LinearProgressIndicator(
                            progress = { row.approvalCount.toFloat() / state.threshold.coerceAtLeast(1u).toFloat() },
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Text("${row.approvalCount}/${state.threshold} approvals")
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Button(onClick = { viewModel.approve(row.id) }, enabled = state.publicKey.isNotEmpty() && !state.isLoading) { Text("Approve") }
                            TextButton(onClick = { viewModel.reject(row.id) }, enabled = state.publicKey.isNotEmpty() && !state.isLoading) { Text("Reject") }
                        }
                    }
                }
            }
        }
        if (state.statusMessage.isNotEmpty()) item { Text(state.statusMessage, style = MaterialTheme.typography.bodySmall) }
        if (state.isLoading) item { CircularProgressIndicator() }
    }
}

@Composable
private fun CreateVaultTab(viewModel: VaultViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.ui.collectAsState()
    var name by remember { mutableStateOf("") }
    var threshold by remember { mutableIntStateOf(2) }
    var includeSelf by remember { mutableStateOf(true) }
    val extras = remember { mutableStateListOf("") }

    val signers = buildList {
        if (includeSelf && state.publicKey.isNotEmpty()) add(state.publicKey)
        addAll(extras.map { it.trim() }.filter { it.isNotEmpty() })
    }.distinct()

    Column(modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("Deploy a new N-of-M vault on-chain via the factory contract.")
        OutlinedTextField(name, { name = it }, label = { Text("Vault name") }, modifier = Modifier.fillMaxWidth())
        Text("Threshold: $threshold-of-${signers.size.coerceAtLeast(1)}")
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(onClick = { if (threshold > 1) threshold-- }) { Text("-") }
            TextButton(onClick = { if (threshold < signers.size.coerceAtLeast(1)) threshold++ }) { Text("+") }
        }
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Switch(includeSelf, { includeSelf = it }, enabled = state.publicKey.isNotEmpty())
            Text("Include me", modifier = Modifier.padding(start = 8.dp))
        }
        extras.forEachIndexed { i, v ->
            OutlinedTextField(v, { extras[i] = it }, label = { Text("Co-signer G…") }, modifier = Modifier.fillMaxWidth())
        }
        TextButton(onClick = { extras.add("") }) { Text("Add co-signer") }
        Button(
            onClick = { viewModel.createVault(name, signers, threshold.toUInt()) },
            enabled = !state.isLoading && name.isNotBlank() && signers.size >= threshold && includeSelf && state.publicKey.isNotEmpty(),
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Create vault on-chain") }
        if (state.statusMessage.isNotEmpty()) Text(state.statusMessage)
    }
}

@Composable
private fun SettingsTab(viewModel: VaultViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.ui.collectAsState()
    var secret by remember { mutableStateOf("") }
    Column(modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        OutlinedTextField(state.vaultAddress, viewModel::updateVaultAddress, label = { Text("Vault") }, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(state.factoryAddress, viewModel::updateFactoryAddress, label = { Text("Factory") }, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(state.rpcUrl, viewModel::updateRpcUrl, label = { Text("RPC URL") }, modifier = Modifier.fillMaxWidth())
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(onClick = { viewModel.updateNetwork("testnet") }) { Text("Testnet") }
            TextButton(onClick = { viewModel.updateNetwork("mainnet") }) { Text("Mainnet") }
        }
        Button(onClick = viewModel::saveSettings) { Text("Save") }
        TextButton(onClick = viewModel::useDemoVault) { Text("Use demo vault") }
        if (state.publicKey.isNotEmpty()) Text("Signer: ${state.publicKey}", style = MaterialTheme.typography.bodySmall)
        OutlinedTextField(secret, { secret = it }, label = { Text("Secret S…") }, visualTransformation = PasswordVisualTransformation(), modifier = Modifier.fillMaxWidth())
        Button(onClick = { viewModel.saveSecret(secret); secret = "" }) { Text("Save to encrypted storage") }
        TextButton(onClick = viewModel::clearSecret) { Text("Remove secret") }
    }
}
