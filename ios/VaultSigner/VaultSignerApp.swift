import SwiftUI

@main
struct VaultSignerApp: App {
    @StateObject private var model = VaultViewModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
        }
    }
}
