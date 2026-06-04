import 'dart:async';
import 'package:flutter/services.dart';
import '../screens/capture_screen.dart';

class ShareIntentService {
  static const _channel = MethodChannel('relay://share');
  static final Set<String> _recentShares = {};
  static const _dedupSeconds = 60;
  static void Function()? onNavigateToCapture;

  static void init() {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'onShare') {
        final text = call.arguments as String? ?? '';
        await _handleShare(text);
      }
    });
  }

  static Future<void> _handleShare(String text) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (_recentShares.contains(_dedupKey(text))) return;

    final relayMatch = RegExp(r'^relay:\/\/stream\/(.+)$').firstMatch(text);
    if (relayMatch != null) return;

    _recentShares.add(_dedupKey(text));
    Timer(const Duration(seconds: _dedupSeconds), () {
      _recentShares.remove(_dedupKey(text));
    });

    CaptureScreen.pendingSharedText = text;
    onNavigateToCapture?.call();
  }

  static String _dedupKey(String text) {
    return text.substring(0, text.length.clamp(0, 100));
  }
}