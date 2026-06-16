import SwiftUI

struct MainTabView: View {
    @EnvironmentObject var model: VaultViewModel

    var body: some View {
        TabView {
            VaultDashboardView()
                .tabItem { Label("Vault", systemImage: "building.columns.fill") }

            CreateVaultView()
                .tabItem { Label("Create", systemImage: "plus.circle.fill") }

            SettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape.fill") }
        }
        .tint(AppTheme.accent)
    }
}

#Preview {
    MainTabView()
        .environmentObject(VaultViewModel())
}
