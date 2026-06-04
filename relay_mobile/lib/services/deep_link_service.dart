import 'package:app_links/app_links.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../services/relay_service.dart';

/// Singleton that listens for `relay://` deep-links and routes the app.
///
/// Call [setNavigatorKey] once during app setup (after defining your
/// `GoRouter`), then call [init] to start listening.
///
/// Supported schemes:
///   relay://stream/{id}     → open stream detail
///   relay://subscribe/{id}  → subscribe to stream
class DeepLinkService {
  static final DeepLinkService _instance = DeepLinkService._();
  factory DeepLinkService() => _instance;
  DeepLinkService._();

  AppLinks? _appLinks;
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();

  /// The [GlobalKey] must be passed to your `GoRouter` as its
  /// `navigatorKey` so that we can navigate without a stale [BuildContext].
  GlobalKey<NavigatorState> get navigatorKey => _navigatorKey;

  BuildContext? get _context => _navigatorKey.currentContext;

  Future<void> init() async {
    _appLinks = AppLinks();

    // Handle link that launched the app (cold start)
    try {
      final uri = await _appLinks!.getInitialLink();
      if (uri != null) _handle(uri);
    } catch (_) {
      // No initial link or permission denied
    }

    // Handle links while app is running
    _appLinks!.uriLinkStream.listen(
      (Uri uri) => _handle(uri),
      onError: (_) {},
    );
  }

  /// Validate a stream ID to prevent injection or malformed data.
  static bool isValidStreamId(String id) {
    return id.isNotEmpty &&
        id.length <= 64 &&
        RegExp(r'^[a-zA-Z0-9_-]+$').hasMatch(id);
  }

  void _handle(Uri uri) {
    if (uri.scheme != 'relay') return;
    final pathSegments = uri.pathSegments;
    if (pathSegments.length < 2) return;

    final action = pathSegments[0];
    final id = pathSegments[1];

    if (!isValidStreamId(id)) {
      debugPrint('Invalid stream ID in deep-link: $id');
      return;
    }

    switch (action) {
      case 'stream':
      case 'subscribe':
        _subscribeAndConfirm(id);
        break;
    }
  }

  Future<void> _subscribeAndConfirm(String streamId) async {
    try {
      await RelayService().subscribeToStream(streamId);
      final ctx = _context;
      if (ctx != null && ctx.mounted) {
        ScaffoldMessenger.of(ctx).showSnackBar(
          SnackBar(content: Text('Subscribed to stream $streamId')),
        );
        ctx.go('/');
      }
    } catch (e) {
      final ctx = _context;
      if (ctx != null && ctx.mounted) {
        ScaffoldMessenger.of(ctx).showSnackBar(
          SnackBar(content: Text('Subscribe failed: $e')),
        );
      }
    }
  }
}
