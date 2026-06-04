import 'dart:async';
import 'dart:convert';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'model_download_service.dart';
import 'download_progress.dart';

/// Configuration for a model download on a given platform.
class ModelConfig {
  final String platform;
  final String modelUrl;
  final String sha256;
  final int expectedSizeBytes;

  const ModelConfig({
    required this.platform,
    required this.modelUrl,
    required this.sha256,
    required this.expectedSizeBytes,
  });

  Map<String, dynamic> toJson() => {
    'platform': platform,
    'modelUrl': modelUrl,
    'sha256': sha256,
    'expectedSizeBytes': expectedSizeBytes,
  };

  factory ModelConfig.fromJson(Map<String, dynamic> json) => ModelConfig(
        platform: json['platform'] as String,
        modelUrl: json['modelUrl'] as String,
        sha256: json['sha256'] as String,
        expectedSizeBytes: json['expectedSizeBytes'] as int,
      );
}

/// Singleton manager that coordinates one active model download at a time and
/// persists download state to SharedPreferences so it survives app restarts.
class ModelDownloadManager {
  static final ModelDownloadManager _instance = ModelDownloadManager._internal();
  factory ModelDownloadManager() => _instance;
  ModelDownloadManager._internal();

  static const _prefsKey = 'relay_model_download_state';

  IoModelDownloadService? _service;
  ModelConfig? _activeConfig;
  DownloadProgress? _lastProgress;
  StreamSubscription<DownloadProgress>? _sub;

  final _statusController = StreamController<DownloadProgress>.broadcast();

  /// Exposes progress updates for the UI.
  Stream<DownloadProgress> get statusStream => _statusController.stream;

  /// Starts a new download, cancelling any active one.
  Future<void> startDownload(ModelConfig config) async {
    cancelDownload();
    _activeConfig = config;
    await _persistState(config);

    _service = IoModelDownloadService();
    _sub = _service!.progressStream.listen((progress) {
      _lastProgress = progress;
      if (!_statusController.isClosed) _statusController.add(progress);
    });

    await _service!.downloadModel(
      url: config.modelUrl,
      sha256: config.sha256,
      savePath: await _resolveSavePath(config),
    );
  }

  /// Pauses the active download by cancelling it. Call [resumeDownload] later.
  void pauseDownload() {
    _service?.cancelDownload();
    _sub?.cancel();
    _updateStatus(DownloadStatus.paused);
  }

  /// Resumes a paused download by re-reading persisted config and starting again.
  Future<void> resumeDownload() async {
    final config = await _restoreConfig();
    if (config != null) {
      await startDownload(config);
    }
  }

  /// Cancels and forgets the active download.
  void cancelDownload() {
    _service?.cancelDownload();
    _sub?.cancel();
    _service = null;
    _activeConfig = null;
  }

  /// Returns the latest known progress or a pending state if none exists.
  Future<DownloadProgress> getDownloadStatus() async {
    if (_lastProgress != null) return _lastProgress!;
    final prefs = await SharedPreferences.getInstance();
    final jsonStr = prefs.getString(_prefsKey);
    if (jsonStr == null) {
      return const DownloadProgress(
        bytesDownloaded: 0,
        totalBytes: 0,
        status: DownloadStatus.pending,
      );
    }
    return const DownloadProgress(
      bytesDownloaded: 0,
      totalBytes: 0,
      status: DownloadStatus.pending,
    );
  }

  /// Whether the model file exists and passes integrity verification.
  Future<bool> isModelReady(ModelConfig config) async {
    final path = await _resolveSavePath(config);
    return IoModelDownloadService().verifyIntegrity(path, config.sha256);
  }

  // -----------------------------------------------------------------------
  // Persistence helpers
  // -----------------------------------------------------------------------

  Future<void> _persistState(ModelConfig config) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_prefsKey, jsonEncode(config.toJson()));
  }

  Future<ModelConfig?> _restoreConfig() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_prefsKey);
    if (raw == null) return null;
    try {
      final map = jsonDecode(raw) as Map<String, dynamic>;
      return ModelConfig.fromJson(map);
    } catch (_) {
      return null;
    }
  }

  Future<String> _resolveSavePath(ModelConfig config) async {
    final dir = await getApplicationSupportDirectory();
    return '${dir.path}/relay/models/${config.platform}/${config.sha256}';
  }

  void _updateStatus(DownloadStatus status) {
    _lastProgress = DownloadProgress(
      bytesDownloaded: _lastProgress?.bytesDownloaded ?? 0,
      totalBytes: _lastProgress?.totalBytes ?? 0,
      status: status,
    );
    if (!_statusController.isClosed) _statusController.add(_lastProgress!);
  }
}
