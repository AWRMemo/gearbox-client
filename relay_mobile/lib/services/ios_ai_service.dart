import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// iOS-specific MLX Swift AI enrichment service.
///
/// Communicates with [AiPlugin.swift] via MethodChannel `com.gearbox.ai`.
class IosAiService {
  static const MethodChannel _channel = MethodChannel('com.gearbox.ai');

  /// Sends [text] to the on-device MLX Swift model and returns a JSON
  /// string containing `tags` and `summary`.
  static Future<String> enrichHighlight(String text) async {
    try {
      final result = await _channel.invokeMethod<String>(
        'enrichHighlight',
        {'text': text},
      );
      if (result == null) {
        throw Exception('IosAiService: null response from channel');
      }
      return result;
    } on PlatformException catch (e) {
      debugPrint('IosAiService error: ${e.code} — ${e.message}');
      rethrow;
    }
  }
}
