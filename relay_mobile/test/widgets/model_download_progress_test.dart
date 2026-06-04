import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:relay_mobile/services/download_progress.dart';
import 'package:relay_mobile/widgets/model_download_progress.dart';

void main() {
  group('ModelDownloadProgress', () {
    late StreamController<DownloadProgress> controller;

    setUp(() {
      controller = StreamController<DownloadProgress>.broadcast();
    });

    tearDown(() async {
      await controller.close();
    });

    testWidgets('renders pending state with correct title',
        (WidgetTester tester) async {
      controller.add(const DownloadProgress(
        bytesDownloaded: 0,
        totalBytes: 0,
        status: DownloadStatus.pending,
      ));

      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: ModelDownloadProgress(statusStream: controller.stream),
        ),
      ));
      await tester.pump();

      expect(find.text('Download pending'), findsOneWidget);
    });

    testWidgets('shows pause button when downloading', (WidgetTester tester) async {
      controller.add(const DownloadProgress(
        bytesDownloaded: 512 * 1024,
        totalBytes: 1024 * 1024,
        status: DownloadStatus.downloading,
      ));

      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: ModelDownloadProgress(statusStream: controller.stream),
        ),
      ));
      await tester.pump();

      expect(find.text('Pause'), findsOneWidget);
    });

    testWidgets('shows completed state without buttons', (WidgetTester tester) async {
      controller.add(const DownloadProgress(
        bytesDownloaded: 1024 * 1024,
        totalBytes: 1024 * 1024,
        status: DownloadStatus.completed,
      ));

      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: ModelDownloadProgress(statusStream: controller.stream),
        ),
      ));
      await tester.pump();

      expect(find.text('Download complete'), findsOneWidget);
      expect(find.text('Cancel'), findsNothing);
      expect(find.text('Pause'), findsNothing);
    });

    testWidgets('updates when stream emits new progress', (WidgetTester tester) async {
      controller.add(const DownloadProgress(
        bytesDownloaded: 0,
        totalBytes: 0,
        status: DownloadStatus.pending,
      ));

      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: ModelDownloadProgress(statusStream: controller.stream),
        ),
      ));
      await tester.pump();

      expect(find.text('Download pending'), findsOneWidget);

      controller.add(const DownloadProgress(
        bytesDownloaded: 1234567,
        totalBytes: 2469134,
        status: DownloadStatus.downloading,
      ));
      await tester.pump();

      expect(find.text('Downloading model'), findsOneWidget);
      expect(find.text('Download pending'), findsNothing);
    });
  });
}
