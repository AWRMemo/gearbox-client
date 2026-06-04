import WidgetKit
import SwiftUI

struct RelayWidgetEntry: TimelineEntry {
    let date: Date
    let captureCount: Int
    let lastSummary: String
}

struct RelayWidgetProvider: TimelineProvider {
    func placeholder(in context: Context) -> RelayWidgetEntry {
        RelayWidgetEntry(date: Date(), captureCount: 0, lastSummary: "No captures yet")
    }

    func getSnapshot(in context: Context, completion: @escaping (RelayWidgetEntry) -> Void) {
        let entry = RelayWidgetEntry(date: Date(), captureCount: 0, lastSummary: "Capture text to get started")
        completion(entry)
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<RelayWidgetEntry>) -> Void) {
        let defaults = UserDefaults(suiteName: "group.com.gearbox.relay")
        let count = defaults?.integer(forKey: "capture_count") ?? 0
        let summary = defaults?.string(forKey: "last_summary") ?? "No captures yet"
        let entry = RelayWidgetEntry(date: Date(), captureCount: count, lastSummary: summary)
        let nextUpdate = Calendar.current.date(byAdding: .minute, value: 15, to: Date())!
        let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
        completion(timeline)
    }
}

struct RelayWidgetEntryView: View {
    var entry: RelayWidgetEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Relay").font(.headline).foregroundColor(.blue)
            Text("\(entry.captureCount) captures").font(.subheadline)
            Text(entry.lastSummary).font(.caption).lineLimit(2)
        }
    }
}

@main
struct RelayWidget: Widget {
    let kind: String = "com.gearbox.relay.widget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: RelayWidgetProvider()) { entry in
            RelayWidgetEntryView(entry: entry)
        }
        .configurationDisplayName("Relay Stats")
        .description("Your capture count and last summary.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}
