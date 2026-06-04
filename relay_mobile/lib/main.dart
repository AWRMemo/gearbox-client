import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'screens/capture_screen.dart';
import 'screens/history_screen.dart';
import 'screens/onboarding_screen.dart';
import 'screens/search_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/subscribe_screen.dart';
import 'screens/stream_editor.dart';
import 'services/deep_link_service.dart';
import 'services/relay_service.dart';
import 'services/push_service.dart';
import 'services/background_sync_service.dart';
import 'services/share_intent_service.dart';
import 'src/rust/frb_generated.dart';
import 'styles/theme.dart';

@pragma('vm:entry-point')
Future<void> _firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  await Firebase.initializeApp();
  final deepLink = message.data['deep_link'];
  if (deepLink != null) {
    debugPrint('Background push: $deepLink');
  }
}

final router = GoRouter(
  initialLocation: '/',
  navigatorKey: DeepLinkService().navigatorKey,
  routes: [
    GoRoute(
      path: '/',
      builder: (BuildContext context, GoRouterState state) => const HomeScreen(),
    ),
    GoRoute(
      path: '/subscribe/:streamId',
      builder: (BuildContext context, GoRouterState state) => SubscribeScreen(
        streamId: state.pathParameters['streamId']!,
      ),
    ),
    GoRoute(
      path: '/publish',
      builder: (BuildContext context, GoRouterState state) => const StreamEditor(),
    ),
  ],
);

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Firebase.initializeApp();
  await RustLib.init();
  await RelayService().init();

  FirebaseMessaging.onBackgroundMessage(_firebaseMessagingBackgroundHandler);

  FirebaseMessaging.instance.getInitialMessage().then((message) {
    if (message != null) {
      final deepLink = message.data['deep_link'];
      if (deepLink != null) {
        debugPrint('Initial push deep-link: $deepLink');
      }
    }
  });

  await PushService.init();

  final telemetryDisabled = await RelayService().getTelemetryOptOut();
  final sentryDsn = const String.fromEnvironment('SENTRY_DSN', defaultValue: '');

  if (!telemetryDisabled && sentryDsn.isNotEmpty) {
    await SentryFlutter.init(
      (options) {
        options.dsn = sentryDsn;
        options.release = 'relay@0.1.0';
        options.environment = const String.fromEnvironment('SENTRY_ENV', defaultValue: 'production');
        options.sendDefaultPii = false;
        options.beforeSend = (event) {
          event.user = null;
          event.request = null;
          event.serverName = null;
          event.tags = const {};
          event.contexts.remove('highlight_text');
          event.contexts.remove('summary');
          event.contexts.remove('stream_title');
          event.extra?.remove('highlight_text');
          event.extra?.remove('summary');
          event.extra?.remove('stream_title');
          return event;
        };
      },
      appRunner: () => runApp(const RelayApp()),
    );
  } else {
    runApp(const RelayApp());
  }
}

class RelayApp extends StatefulWidget {
  const RelayApp({super.key});

  @override
  State<RelayApp> createState() => _RelayAppState();
}

class _RelayAppState extends State<RelayApp> {
  bool? _onboardingSeen;
  ThemeMode _themeMode = ThemeMode.system;
  StreamSubscription<RemoteMessage>? _messageSubscription;

  @override
  void initState() {
    super.initState();
    _checkOnboarding();
    _loadTheme();
    DeepLinkService().init();
    ShareIntentService.onNavigateToCapture = () => router.go('/');
    ShareIntentService.init();

    _messageSubscription = FirebaseMessaging.onMessageOpenedApp.listen((message) {
      final deepLink = message.data['deep_link'];
      if (deepLink != null) {
        final uri = Uri.tryParse(deepLink);
        if (uri != null &&
            uri.scheme == 'relay' &&
            (uri.host == 'stream' || uri.host == 'subscribe')) {
          final pathSegments = uri.pathSegments;
          if (pathSegments.isNotEmpty) {
            final streamId = pathSegments.last;
            if (DeepLinkService.isValidStreamId(streamId) && mounted) {
              router.go('/subscribe/$streamId');
            }
          }
        }
      }
    });
  }

  Future<void> _checkOnboarding() async {
    final prefs = await SharedPreferences.getInstance();
    setState(() => _onboardingSeen = prefs.getBool('relay_onboarding_seen') ?? false);
    _checkBackgroundSync(prefs);
  }

  Future<void> _checkBackgroundSync(SharedPreferences prefs) async {
    final needsSync = prefs.getBool('flutter.needs_background_sync') ?? false;
    if (needsSync) {
      await prefs.setBool('flutter.needs_background_sync', false);
      BackgroundSyncService.performSync();
    }
  }

  Future<void> _loadTheme() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getString(RelayTheme.storageKey);
    setState(() => _themeMode = RelayTheme.storedMode(stored));
  }

  void _finishOnboarding() {
    setState(() => _onboardingSeen = true);
  }

  @override
  void dispose() {
    _messageSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      routerConfig: router,
      theme: RelayTheme.lightTheme,
      darkTheme: RelayTheme.darkTheme,
      themeMode: _themeMode,
      builder: (context, child) {
        if (_onboardingSeen == null) {
          return const Scaffold(body: Center(child: CircularProgressIndicator()));
        }
        if (!_onboardingSeen!) {
          return OnboardingScreen(onDone: _finishOnboarding);
        }
        return child ?? const SizedBox.shrink();
      },
    );
  }
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _currentIndex = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: const [
          CaptureScreen(),
          HistoryScreen(),
          SearchScreen(),
          SettingsScreen(),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (int index) => setState(() => _currentIndex = index),
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.add_circle_outline),
            selectedIcon: Icon(Icons.add_circle),
            label: 'Capture',
          ),
          NavigationDestination(
            icon: Icon(Icons.history_outlined),
            selectedIcon: Icon(Icons.history),
            label: 'History',
          ),
          NavigationDestination(
            icon: Icon(Icons.search_outlined),
            selectedIcon: Icon(Icons.search),
            label: 'Search',
          ),
          NavigationDestination(
            icon: Icon(Icons.settings_outlined),
            selectedIcon: Icon(Icons.settings),
            label: 'Settings',
          ),
        ],
      ),
    );
  }
}
