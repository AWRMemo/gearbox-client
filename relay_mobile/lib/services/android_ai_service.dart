import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// Android-specific MediaPipe LLM Inference service.
///
/// Communicates via two channels:
/// – [MethodChannel] (`com.gearbox.ai`) for commands
/// – [EventChannel] (`com.gearbox.ai.download`) for download progress
class AndroidAiService {
  static const MethodChannel _methodChannel = MethodChannel('com.gearbox.ai');
  static const EventChannel _eventChannel = EventChannel('com.gearbox.ai.download');

  static Stream<Map<String, dynamic>>? _downloadProgressStream;
  static StreamSubscription<Map<String, dynamic>>? _downloadProgressSubscription;

  /// Current model status: `not_downloaded` | `downloading` | `ready` | `error`.
  static Future<String> getModelStatus() async {
    try {
      final result = await _methodChannel.invokeMethod<String>('getModelStatus');
      return result ?? 'unknown';
    } on PlatformException catch (e) {
      throw _RelayException('getModelStatus', e.code, e.message);
    }
  }

  /// Starts a foreground-service download of the SLM.
  ///
  /// [url] defaults to the CDN endpoint if null.
  static Future<void> startModelDownload({String? url}) async {
    try {
      await _methodChannel.invokeMethod(
        'startModelDownload',
        {'url': url},
      );
    } on PlatformException catch (e) {
      throw _RelayException('startModelDownload', e.code, e.message);
    }
  }

  /// Notifies the native side that the model file is fully available.
  static Future<void> setModelPath(String path) async {
    try {
      await _methodChannel.invokeMethod('setModelPath', {'path': path});
    } on PlatformException catch (e) {
      throw _RelayException('setModelPath', e.code, e.message);
    }
  }

  /// Sends [text] to the on-device MediaPipe model and returns a JSON
  /// string containing `tags`, `summary`, and `connection_suggestion`.
  static Future<String> enrichHighlight(String text) async {
    try {
      final result = await _methodChannel.invokeMethod<String>(
        'enrichHighlight',
        {'text': text},
      );
      if (result == null) {
        throw const _RelayException.enrichmentMissing();
      }
      return result;
    } on PlatformException catch (e) {
      debugPrint('AndroidAiService error: ${e.code} — ${e.message}');
      throw _RelayException('enrichHighlight', e.code, e.message);
    }
  }

  /// Returns a persistent [Stream] of download-progress events.
  /// Each event is a map with keys `downloaded` (int), `total` (int), `status` (String).
  static Stream<Map<String, dynamic>> downloadProgress() {
    _downloadProgressStream ??= _eventChannel
        .receiveBroadcastStream()
        .map((dynamic event) {
          final map = Map<String, dynamic>.from(event as Map);
          return map;
        })
        .handleError((Object error) {
          debugPrint('Download event stream error: $error');
        });
    return _downloadProgressStream!;
  }

  /// Cancels the active download-progress listener.
  static Future<void> dispose() async {
    await _downloadProgressSubscription?.cancel();
    _downloadProgressSubscription = null;
  }
}

/// Structured error type for Android AI operations.
class _RelayException implements Exception {
  final String method;
  final String code;
  final String? message;

  const _RelayException(this.method, this.code, this.message);

  const _RelayException.enrichmentMissing()
      : method = 'enrichHighlight',
        code = 'NULL_RESPONSE',
        message = 'Native side returned null for enrichment result';

  @override
  String toString() => '_RelayException(method=$method, code=$code, message=$message)';
}
