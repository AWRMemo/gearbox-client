import Foundation
import MLXLLM
import MLXLMCommon
import MLXHuggingFace
import HuggingFace
import Tokenizers

/// MlxBridge exposes a single synchronous-style API for the Flutter side:
/// load a Qwen-3.5-0.8B model from Hugging Face and run a prompt.
public enum MlxBridge {
    private static var modelContainer: ModelContainer?
    private static let lock = NSLock()

    /// Load the model lazily on first call.
    private static func ensureLoaded() async throws -> ModelContainer {
        lock.lock()
        defer { lock.unlock() }
        if let container = modelContainer {
            return container
        }
        let config = ModelConfiguration(
            id: "mlx-community/Qwen3.5-0.8B-OptiQ-4bit",
            defaultPrompt: ""
        )
        let container = try await LLMModelFactory.shared.loadContainer(
            from: #hubDownloader(),
            using: #huggingFaceTokenizerLoader(),
            configuration: config
        )
        modelContainer = container
        return container
    }

    /// Called by the app on memory warning.
    public static func releaseModel() {
        lock.lock()
        defer { lock.unlock() }
        modelContainer = nil
    }

    /// Returns a JSON-string `{"tags": [...], "summary": "..."}`.
    public static func enrichHighlight(text: String) async throws -> String {
        let container = try await ensureLoaded()
        let session = ChatSession(container)
        let prompt = buildPrompt(for: text)
        let response = try await session.respond(to: prompt)
        return parseToJson(response.text)
    }

    // MARK: - Prompt / parsing helpers

    private static func buildPrompt(for text: String) -> String {
        let escaped = text
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        return """
        You are a personal knowledge assistant. Given the user highlight below, produce valid JSON with exactly two keys: "summary" (a one-sentence summary) and "tags" (an array of 3-5 lowercase keyword strings).
        
        Highlight: "\(escaped)"
        
        Respond ONLY with JSON. No markdown fences.
        """
    }

    private static func parseToJson(_ raw: String) -> String {
        var trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)

        // Layer 1: strip markdown fences
        if trimmed.hasPrefix("```json"),
           let end = trimmed.range(of: "```",
                                   options: [],
                                   range: trimmed.index(trimmed.startIndex, offsetBy: 7)..<trimmed.endIndex) {
            let sliceStart = trimmed.index(trimmed.startIndex, offsetBy: 7)
            trimmed = String(trimmed[sliceStart..<end.lowerBound]).trimmingCharacters(in: .whitespacesAndNewlines)
        } else if trimmed.hasPrefix("```"),
                  let end = trimmed.range(of: "```",
                                          options: [],
                                          range: trimmed.index(trimmed.startIndex, offsetBy: 3)..<trimmed.endIndex) {
            let sliceStart = trimmed.index(trimmed.startIndex, offsetBy: 3)
            trimmed = String(trimmed[sliceStart..<end.lowerBound]).trimmingCharacters(in: .whitespacesAndNewlines)
        }

        // Layer 2: validate JSON object
        if trimmed.hasPrefix("{"), trimmed.hasSuffix("}") {
            // Reject degenerate outputs (empty tags + empty summary)
            if !trimmed.contains("\u0022tags\u0022:[\u0022") && !trimmed.contains("\u0022summary\u0022:\u0022\u0022") {
                return trimmed
            }
        }

        // Layer 3: fallback keyword extraction
        let tags = extractTags(from: raw)
        let summary = extractSummary(from: raw)
        return "{\"tags\":\(tags),\"summary\":\"\(summary)\",\"connection_suggestion\":null}"
    }

    private static func extractTags(from text: String) -> String {
        let pattern = #""tags"\s*:\s*\[([^\]]*)\]"#
        if let range = text.range(of: pattern, options: .regularExpression) {
            let match = String(text[range])
            if let bracketStart = match.firstIndex(of: "["),
               let bracketEnd = match.lastIndex(of: "]") {
                let inner = String(match[bracketStart...bracketEnd])
                return inner
            }
        }
        return "[\"spike\"]"
    }

    private static func extractSummary(from text: String) -> String {
        let pattern = #""summary"\s*:\s*"([^"]+)""#
        if let range = text.range(of: pattern, options: .regularExpression) {
            let match = String(text[range])
            if let start = match.firstIndex(of: "\""),
               let end = match.lastIndex(of: "\"") {
                let after = match.index(after: start)
                let before = match.index(before: end)
                let inner = String(match[after...before])
                    .replacingOccurrences(of: "\\\"", with: "\"")
                return inner
            }
        }
        return text.prefix(120)
            .replacingOccurrences(of: "\"", with: "\\\"")
    }
}
