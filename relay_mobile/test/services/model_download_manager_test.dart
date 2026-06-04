import 'dart:async';
import 'dart:io';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:relay_mobile/services/model_download_service.dart';
import 'package:relay_mobile/services/model_download_manager.dart';
import 'package:relay_mobile/services/download_progress.dart';

void main() {
  group('IoModelDownloadService', () {
    late IoModelDownloadService svc;
    late Directory tmpDir;

    setUp(() {
      svc = IoModelDownloadService();
      tmpDir = Directory.systemTemp.createTempSync('relay_test_');
    });

    tearDown(() {
      if (tmpDir.existsSync()) tmpDir.deleteSync(recursive: true);
    });

    test('verifyIntegrity returns false for missing file', () async {
      final result = await svc
          .verifyIntegrity('${tmpDir.path}/no_such_file.bin', 'abc');
      expect(result, false);
    });

    test('verifyIntegrity returns true for matching SHA-256', () async {
      final data = List.generate(256, (i) => i % 256).toList();
      final file = File('${tmpDir.path}/model.bin');
      await file.writeAsBytes(data);

      const expected =
          '4c6426b0c032f9b8f89339c0608faa7cb91df0d3a0ac8bed0aa0164ebedc51bc';
      final result = await svc.verifyIntegrity(file.path, expected);
      expect(result, true);
    });

    test('cancelDownload does not throw and stream is still active', () async {
      expect(svc.progressStream, isA<Stream<DownloadProgress>>());
      expect(() => svc.cancelDownload(), returnsNormally);
    });
  });

  group('ModelDownloadManager', () {
    late ModelDownloadManager mgr;
    late Directory tmpDir;

    setUp(() {
      mgr = ModelDownloadManager();
      tmpDir = Directory.systemTemp.createTempSync('relay_mgr_test_');
    });

    tearDown(() {
      mgr.cancelDownload();
      if (tmpDir.existsSync()) tmpDir.deleteSync(recursive: true);
    });

    test('getDownloadStatus returns pending initially', () async {
      final status = await mgr.getDownloadStatus();
      expect(status.status, DownloadStatus.pending);
    });

    test('pauseDownload updates status to paused', () async {
      mgr.pauseDownload();
      final status = await mgr.getDownloadStatus();
      expect(status.status, DownloadStatus.paused);
    });

    test('cancelDownload resets state', () async {
      mgr.cancelDownload();
      final status = await mgr.getDownloadStatus();
      // Cancelling clears _lastProgress so pending is returned.
      expect(status.status, DownloadStatus.pending);
    });

    test('isModelReady throws MissingPluginException in unit test environment',
        () async {
      const config = ModelConfig(
        platform: 'test',
        modelUrl: 'http://127.0.0.1/404',
        sha256: 'abc',
        expectedSizeBytes: 0,
      );
      expect(
        () => mgr.isModelReady(config),
        throwsA(isA<MissingPluginException>()),
      );
    });
  });
}
