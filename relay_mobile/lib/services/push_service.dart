import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'relay_service.dart';

class PushService {
  static final FlutterLocalNotificationsPlugin _localPlugin =
      FlutterLocalNotificationsPlugin();
  static final SelectNotificationCallback _onTap = (String? payload) {
    if (payload == null) return;
    try {
      final data = jsonDecode(payload) as Map<String, dynamic>;
      final deepLink = data['deep_link'] as String?;
      if (deepLink != null) {
        debugPrint('Local notification tapped: $deepLink');
        // Navigation is handled by DeepLinkService listening to app_links;
        // if the payload was also a URI link we'll route. Otherwise a manual
        // router push is needed from a BuildContext-aware widget.
      }
    } catch (_) {}
  };

  static Future<void> init() async {
    final messaging = FirebaseMessaging.instance;

    final settings = await messaging.requestPermission(
      alert: true, badge: true, sound: true,
    );

    if (settings.authorizationStatus != AuthorizationStatus.authorized) {
      return;
    }

    const channel = AndroidNotificationChannel(
      'high_importance_channel',
      'High Importance Notifications',
      importance: Importance.max,
    );

    await _localPlugin
        .resolvePlatformSpecificImplementation<
            AndroidFlutterLocalNotificationsPlugin>()
        ?.createNotificationChannel(channel);

    await _localPlugin.initialize(
      const InitializationSettings(
        android: AndroidInitializationSettings('@mipmap/ic_launcher'),
      ),
      onDidReceiveNotificationResponse: (details) => _onTap(details.payload),
    );

    // Handle foreground messages
    FirebaseMessaging.onMessage.listen((message) {
      final notification = message.notification;
      if (notification != null) {
        _localPlugin.show(
          message.hashCode,
          notification.title,
          notification.body,
          NotificationDetails(
            android: AndroidNotificationDetails(
              channel.id, channel.name,
              channelDescription: channel.description,
              icon: '@mipmap/ic_launcher',
            ),
          ),
          payload: jsonEncode(message.data),
        );
      }
    });

    FirebaseMessaging.onBackgroundMessage(_backgroundHandler);
    FirebaseMessaging.instance.onTokenRefresh.listen((token) => _register(token));

    final initialToken = await messaging.getToken();
    if (initialToken != null) await _register(initialToken);
  }

  static Future<void> _register(String token) async {
    try {
      await RelayService().registerDeviceToken(token);
    } catch (_) {}
  }
}

@pragma('vm:entry-point')
Future<void> _backgroundHandler(RemoteMessage message) async {
  debugPrint('Background FCM: ${message.data}');
}

typedef SelectNotificationCallback = void Function(String? payload);
