import 'dart:convert';
import 'dart:io' show Platform;
import 'package:flutter/foundation.dart';
import 'package:relay_mobile/models/enrichment_output.dart';
import 'android_ai_service.dart';
import 'ios_ai_service.dart';
import 'relay_service.dart';
import 'enrichment_parser.dart';

/// Model-agnostic AI enrichment service returning typed output.
///
/// On Android, delegates to [AndroidAiService] backed by MediaPipe.
/// On iOS, delegates to [IosAiService] backed by MLX Swift.
/// On other platforms (desktop / web), falls back to the Rust bridge
/// via [RelayService].
class AiService {
  static final AiService _instance = AiService._();
  factory AiService() => _instance;
  AiService._();

  /// Enriches a highlight text with tags and summary.
  /// Returns a structured [EnrichmentOutput].
  Future<EnrichmentOutput> enrichHighlight(String text) async {
    try {
      final rawJson = await _enrichRaw(text);
      return EnrichmentParser.parse(rawJson);
    } catch (e, stack) {
      if (kDebugMode) {
        debugPrint('AiService.enrichHighlight failed: $e\n$stack');
      }
      // Always return a valid output — never crash the capture flow.
      return EnrichmentParser.parse(text); // fallback parser on raw text
    }
  }

  /// Queries the current model readiness status.
  ///
  /// - Android: `not_downloaded` | `downloading` | `ready` | `error`
  /// - iOS: `ready` (model loads on first use) or `error`
  /// - Desktop: `ready` (Rust bridge always available)
  Future<ModelStatus> getModelStatus() async {
    if (Platform.isAndroid && !kIsWeb) {
      return AndroidAiService.getModelStatus();
    }
    if (Platform.isIOS && !kIsWeb) {
      // iOS model loads lazily from HF; we report ready unless load fails
      return ModelStatus.ready;
    }
    return ModelStatus.ready; // Desktop Rust bridge
  }

  /// Android-only: stream of download progress events.
  Stream<DownloadProgress>? get modelDownloadProgress {
    if (Platform.isAndroid && !kIsWeb) {
      return AndroidAiService.downloadProgress;
    }
    return null;
  }

  Future<String> _enrichRaw(String text) async {
    if (Platform.isAndroid && !kIsWeb) {
      return AndroidAiService.enrichHighlight(text);
    }
    if (Platform.isIOS && !kIsWeb) {
      return IosAiService.enrichHighlight(text);
    }
    // Fallback to Rust bridge (llama-cpp-2 on desktop)
    final result = await RelayService().enrichAndStore(text);
    final encoded = jsonEncode({
      'tags': result.tags,
      'summary': result.summary,
      if (result.suggestionHighlightId != null && result.suggestionHighlightId!.isNotEmpty)
        'connection_suggestion': {
          'source_highlight_id': result.suggestionHighlightId,
          'bridging_sentence': result.suggestionBridgingSentence,
        },
    });
    return encoded;
  }
}

enum ModelStatus { notDownloaded, downloading, ready, error }

class DownloadProgress {
  final int bytesDownloaded;
  final int totalBytes;
  final String status; // pending, downloading, paused, completed, failed
  final String? errorMessage;

  const DownloadProgress({
    required this.bytesDownloaded,
    required this.totalBytes,
    required this.status,
    this.errorMessage,
  });
}
