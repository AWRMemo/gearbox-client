import 'package:flutter_test/flutter_test.dart';
import 'package:relay_mobile/models/enrichment_output.dart';
import 'package:relay_mobile/services/enrichment_parser.dart';

void main() {
  group('EnrichmentParser', () {
    // ---------- Well-Formed JSON Cases ----------
    test('parses canonical JSON with tags and summary', () {
      final raw = '{"tags":["rust","memory","safety"],"summary":"Rust prevents data races at compile time."}';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['rust', 'memory', 'safety']);
      expect(out.summary, 'Rust prevents data races at compile time.');
      expect(out.connectionSuggestion, isNull);
    });

    test('parses JSON with connection_suggestion', () {
      final raw = '{"tags":["rust"],"summary":"...","connection_suggestion":{"source_highlight_id":"hl_abc","bridging_sentence":"Both use ownership."}}';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['rust']);
      expect(out.connectionSuggestion, isNotNull);
      expect(out.connectionSuggestion!.sourceHighlightId, 'hl_abc');
      expect(out.connectionSuggestion!.bridgingSentence, 'Both use ownership.');
    });

    test('parses JSON wrapped in markdown fences', () {
      final raw = '```json\n{"tags":["ai","ml"],"summary":"Machine learning is evolving."}\n```';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['ai', 'ml']);
      expect(out.summary, 'Machine learning is evolving.');
    });

    test('parses JSON with null connection_suggestion', () {
      final raw = '{"tags":["llm"],"summary":"Test.","connection_suggestion":null}';
      final out = EnrichmentParser.parse(raw);
      expect(out.connectionSuggestion, isNull);
    });

    test('parses JSON with extra whitespace and newlines', () {
      final raw = '\n  {"tags": ["test"], "summary": "Summary."}\n';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['test']);
      expect(out.summary, 'Summary.');
    });

    // ---------- Malformed JSON Cases ----------
    test('falls back on missing closing brace', () {
      final raw = '{"tags":["oops"],"summary":"broken"'; // no closing brace
      final out = EnrichmentParser.parse(raw);
      expect(out.tags.length, greaterThanOrEqualTo(1));
      expect(out.summary, isNotEmpty);
    });

    test('falls back on non-JSON output', () {
      final raw = 'Here are some tags: rust, memory. Summary: Safety without GC.';
      final out = EnrichmentParser.parse(raw);
      // Should extract keywords from raw text
      expect(out.tags, isNotEmpty);
      expect(out.summary, isNotEmpty);
    });

    test('falls back on XML-wrapped JSON', () {
      final raw = '<json>{"tags":["xml"],"summary":"XML wrapped."}</json>';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['xml']);
      expect(out.summary, 'XML wrapped.');
    });

    test('falls back on array-outer JSON', () {
      final raw = '[{"tags":["array"],"summary":"Array outer."}]';
      final out = EnrichmentParser.parse(raw);
      expect(out.tags, ['array']);
      expect(out.summary, 'Array outer.');
    });

    // ---------- Edge Cases ----------
    test('handles empty tags and empty summary', () {
      final raw = '{"tags":[],"summary":""}';
      final out = EnrichmentParser.parse(raw);
      // Strict deser rejects empty fields, so fallback kicks in
      expect(out.tags, isNotEmpty);
      expect(out.summary, isNotEmpty);
    });

    test('deals gracefully with truly garbage input', () {
      final raw = '<<<<invalid>>>>';
      final out = EnrichmentParser.parse(raw);
      // Should fallback on raw text tokenisation
      expect(out.tags, ['general']);
      expect(out.summary, isNotEmpty);
    });
  });
}
