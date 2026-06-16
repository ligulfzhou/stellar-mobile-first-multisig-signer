import SwiftUI

enum AppTheme {
    static let accent = Color(red: 0.49, green: 0.23, blue: 0.93)
    static let accentSecondary = Color(red: 0.35, green: 0.55, blue: 0.98)
    static let cardBackground = Color(.secondarySystemGroupedBackground)
    static let heroGradient = LinearGradient(
        colors: [accent, accentSecondary],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )
}

struct CardModifier: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding()
            .background(AppTheme.cardBackground)
            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}

extension View {
    func vaultCard() -> some View {
        modifier(CardModifier())
    }
}
