import 'dart:async';
import 'dart:io';
import 'package:convert/convert.dart';
import 'package:crypto/crypto.dart';
import 'download_progress.dart';

/// Abstract interface for a platform-agnostic model download service.
abstract class ModelDownloadService {
  /// Initiates a download of the model from [url] to [savePath] and verifies
  /// it against [sha256]. Progress is emitted on [progressStream].
  Future<void> downloadModel({
    required String url,
    required String sha256,
    required String savePath,
  });

  /// Progress events for the active (or most recent) download.
  Stream<DownloadProgress> get progressStream;

  /// Verifies the file at [path] matches [expectedSha256] using SHA-256.
  Future<bool> verifyIntegrity(String path, String expectedSha256);

  /// Cancels the ongoing download immediately.
  void cancelDownload();
}

/// dart:io implementation using HttpClient + RandomAccessFile for resumable
/// downloads. Supports exponential backoff (0 s, 5 s, 30 s) and cancellation.
class IoModelDownloadService implements ModelDownloadService {
  final _progressController = StreamController<DownloadProgress>.broadcast();
  HttpClient? _client;
  RandomAccessFile? _file;
  HttpClientRequest? _request;
  HttpClientResponse? _response;
  bool _cancelled = false;

  static const _retryDelays = [
    Duration.zero,
    Duration(seconds: 5),
    Duration(seconds: 30),
  ];

  @override
  Stream<DownloadProgress> get progressStream => _progressController.stream;

  @override
  Future<void> downloadModel({
    required String url,
    required String sha256,
    required String savePath,
  }) async {
    _cancelled = false;
    final uri = Uri.parse(url);
    int bytesAlready = 0;
    final file = File(savePath);
    if (file.existsSync()) {
      bytesAlready = file.lengthSync();
      if (await verifyIntegrity(savePath, sha256)) {
        _emit(bytesAlready, bytesAlready, DownloadStatus.completed);
        return;
      }
    }

    for (var attempt = 0; attempt < _retryDelays.length; attempt++) {
      if (_cancelled) break;
      if (attempt > 0) {
        _emit(bytesAlready, 0, DownloadStatus.pending,
            errorMessage: 'Retrying in ${_retryDelays[attempt].inSeconds}s…');
        await Future.delayed(_retryDelays[attempt]);
      }
      try {
        await _attemptDownload(uri, savePath, bytesAlready);
        final ok = await verifyIntegrity(savePath, sha256);
        if (ok) {
          final len = File(savePath).lengthSync();
          _emit(len, len, DownloadStatus.completed);
          return;
        } else {
          _emit(0, 0, DownloadStatus.failed,
              errorMessage: 'Integrity check failed');
        }
      } on _DownloadCancelled {
        _emit(bytesAlready, 0, DownloadStatus.paused);
        return;
      } catch (e) {
        _emit(bytesAlready, 0, DownloadStatus.failed, errorMessage: e.toString());
      }
    }
    _emit(0, 0, DownloadStatus.failed,
        errorMessage: 'Max retry attempts exceeded');
  }

  Future<void> _attemptDownload(Uri uri, String savePath, int resumeFrom) async {
    _client = HttpClient();
    _request = await _client!.getUrl(uri);
    if (resumeFrom > 0) {
      _request!.headers.add('Range', 'bytes=$resumeFrom-');
    }
    _response = await _request!.close();
    final totalBytes = _resolveTotalBytes(_response!, resumeFrom);

    _file = await File(savePath)
        .open(mode: resumeFrom > 0 ? FileMode.append : FileMode.write);
    int received = resumeFrom;

    await for (final chunk in _response!) {
      if (_cancelled) throw _DownloadCancelled();
      _file!.writeFromSync(chunk);
      received += chunk.length;
      _emit(received, totalBytes, DownloadStatus.downloading);
    }
    await _file!.close();
  }

  int _resolveTotalBytes(HttpClientResponse response, int resumeFrom) {
    final contentLength = response.contentLength;
    if (contentLength > 0) return resumeFrom + contentLength;
    if (resumeFrom > 0) return resumeFrom;
    return 0;
  }

  void _emit(int bytes, int total, DownloadStatus status,
      {String? errorMessage}) {
    if (!_progressController.isClosed) {
      _progressController.add(DownloadProgress(
        bytesDownloaded: bytes,
        totalBytes: total,
        status: status,
        errorMessage: errorMessage,
      ));
    }
  }

  @override
  Future<bool> verifyIntegrity(String path, String expectedSha256) async {
    final file = File(path);
    if (!file.existsSync()) return false;
    final sink = AccumulatorSink<Digest>();
    final input = sha256.startChunkedConversion(sink);
    await for (final chunk in file.openRead()) {
      input.add(chunk);
    }
    input.close();
    final digest = sink.events.single;
    return hex.encode(digest.bytes).toLowerCase() ==
        expectedSha256.toLowerCase();
  }

  @override
  void cancelDownload() {
    _cancelled = true;
    _request?.abort();
    _client?.close(force: true);
    _file?.closeSync();
  }
}

class _DownloadCancelled implements Exception {}
