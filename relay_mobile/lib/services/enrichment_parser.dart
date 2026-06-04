import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../models/enrichment_output.dart';

/// Multi-layer defensive JSON parser for SLM enrichment output.
///
/// Matches desktop parity (`llama_service.rs`) with these layers:
/// 1. Strip markdown fences.
/// 2. Extract first `{...}` object via brace-depth + string-state machine.
/// 3. `jsonDecode` into `EnrichmentOutput` (strict field validation).
/// 4. Loose field extraction with null-safety.
/// 5. Deterministic keyword fallback if all layers fail.
class EnrichmentParser {
  /// Parses [raw] into an [EnrichmentOutput].
  ///
  /// Throws [EnrichmentParseException] only when every layer fails,
  /// which is extremely rare because Layer 5 always produces output.
  static EnrichmentOutput parse(String raw) {
    // Layer 1 — strip markdown fences.
    var cleaned = _stripMarkdownFences(raw);

    // Layer 2 — extract first JSON object.
    final jsonBlock = _extractFirstJsonObject(cleaned);
    if (jsonBlock != null) {
      cleaned = jsonBlock;
    }

    // Layer 3 — strict deserialization.
    try {
      final map = jsonDecode(cleaned) as Map<String, dynamic>;
      final output = EnrichmentOutput.fromJson(map);
      // Reject empty tags + empty summary (degenerate output).
      if (output.tags.isNotEmpty || output.summary.isNotEmpty) {
        return output;
      }
    } catch (e, stack) {
      if (kDebugMode) {
        debugPrint('EnrichmentParser strict deser failed: $e\n$stack');
      }
    }

    // Layer 4 — loose field extraction.
    try {
      final loose = _looseExtract(cleaned);
      if (loose != null) {
        return loose;
      }
    } catch (e, stack) {
      if (kDebugMode) {
        debugPrint('EnrichmentParser loose extraction failed: $e\n$stack');
      }
    }

    // Layer 5 — deterministic keyword fallback.
    return _fallback(raw);
  }

  // -----------------------------------------------------------------
  // Layer 1: strip markdown fences
  // -----------------------------------------------------------------
  static String _stripMarkdownFences(String s) {
    var t = s.trim();
    const fencePatterns = [
      ('```json', '```'),
      ('```', '```'),
      ('`', '`'),
    ];
    for (final (start, end) in fencePatterns) {
      if (t.startsWith(start)) {
        final idx = t.indexOf(end, start.length);
        if (idx != -1) {
          t = t.substring(start.length, idx).trim();
          return t;
        }
      }
    }
    return t;
  }

  // -----------------------------------------------------------------
  // Layer 2: brace-depth + string-state extraction
  // -----------------------------------------------------------------
  static String? _extractFirstJsonObject(String s) {
    var depth = 0;
    var inString = false;
    var escape = false;
    String? start;

    for (var i = 0; i < s.length; i++) {
      final ch = s[i];
      if (escape) {
        escape = false;
        continue;
      }
      if (ch == '\\') {
        escape = true;
        continue;
      }
      if (ch == '"') {
        inString = !inString;
        continue;
      }
      if (!inString) {
        if (ch == '{') {
          if (depth == 0) start = i.toString();
          depth++;
        } else if (ch == '}') {
          depth--;
          if (depth == 0 && start != null) {
            return s.substring(int.parse(start), i + 1);
          }
        }
      }
    }
    return null;
  }

  // -----------------------------------------------------------------
  // Layer 4: loose field extraction
  // -----------------------------------------------------------------
  static EnrichmentOutput? _looseExtract(String s) {
    // Simple regex-based extraction for when JSON is malformed.
    final tagsMatch = RegExp(r'"tags"\s*:\s*(\[[^\]]*\])').firstMatch(s);
    final summaryMatch = RegExp(r'"summary"\s*:\s*"((?:[^"\\]|\\.)*)"').firstMatch(s);

    if (tagsMatch == null && summaryMatch == null) return null;

    List<String> tags = [];
    if (tagsMatch != null) {
      final raw = tagsMatch.group(1)!;
      // Split by quoted strings
      final quoted = RegExp(r'"((?:[^"\\]|\\.)*)"');
      for (final m in quoted.allMatches(raw)) {
        final tag = m.group(1)!.replaceAll('\\"', '"').trim();
        if (tag.isNotEmpty) tags.add(tag);
      }
    }

    var summary = '';
    if (summaryMatch != null) {
      summary = summaryMatch.group(1)!.replaceAll('\\"', '"').trim();
    }

    if (tags.isEmpty && summary.isEmpty) return null;

    return EnrichmentOutput(
      tags: tags.isEmpty ? ['general'] : tags,
      summary: summary.isEmpty ? 'No summary available.' : summary,
      connectionSuggestion: null,
    );
  }

  // -----------------------------------------------------------------
  // Layer 5: deterministic keyword fallback
  // -----------------------------------------------------------------
  static EnrichmentOutput _fallback(String originalText) {
    // Tokenise the original highlight text (not JSON) for keywords.
    final words = originalText
        .toLowerCase()
        .split(RegExp(r'[^a-zA-Z0-9\u00C0-\u024F]+'))
        .where((w) => w.length > 4)
        .toList();
    final tagSet = <String>{};
    const stopWords = {
      'about', 'above', 'after', 'again', 'against', 'could', 'would', 'should',
      'there', 'their', 'which', 'while', 'where', 'being', 'every', 'other',
      'these', 'those', 'what', 'when', 'with', 'your', 'this', 'that', 'from',
      'they', 'have', 'than', 'also', 'were', 'been', 'them', 'into', 'more',
      'very', 'only', 'over', 'such', 'most', 'then', 'well', 'some', 'time',
      'before'
    };
    for (final w in words) {
      if (!stopWords.contains(w)) tagSet.add(w);
      if (tagSet.length >= 5) break;
    }

    final summary = originalText.length > 120
        ? '${originalText.substring(0, 120).trim()}...'
        : originalText.trim();

    return EnrichmentOutput(
      tags: tagSet.toList().isEmpty ? ['general'] : tagSet.toList(),
      summary: summary.isEmpty ? 'No summary available.' : summary,
      connectionSuggestion: null,
    );
  }
}

class EnrichmentParseException implements Exception {
  final String message;
  EnrichmentParseException(this.message);

  @override
  String toString() => 'EnrichmentParseException: $message';
}
