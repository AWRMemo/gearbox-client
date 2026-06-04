import 'dart:async';
import 'package:flutter/material.dart';
import '../services/download_progress.dart';
import '../services/model_download_manager.dart';

/// Circular + linear progress indicator for model downloads.
/// Context-aware buttons: Pause, Resume, Cancel, Retry.
class ModelDownloadProgress extends StatefulWidget {
  final Stream<DownloadProgress>? statusStream;

  const ModelDownloadProgress({super.key, this.statusStream});

  @override
  State<ModelDownloadProgress> createState() => _ModelDownloadProgressState();
}

class _ModelDownloadProgressState extends State<ModelDownloadProgress> {
  DownloadProgress _progress = const DownloadProgress(
    bytesDownloaded: 0,
    totalBytes: 0,
    status: DownloadStatus.pending,
  );
  StreamSubscription<DownloadProgress>? _sub;

  @override
  void initState() {
    super.initState();
    _attach(widget.statusStream ?? ModelDownloadManager().statusStream);
  }

  @override
  void didUpdateWidget(covariant ModelDownloadProgress oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.statusStream != oldWidget.statusStream) {
      _sub?.cancel();
      _attach(widget.statusStream ?? ModelDownloadManager().statusStream);
    }
  }

  void _attach(Stream<DownloadProgress> stream) {
    _sub = stream.listen((p) {
      if (mounted) setState(() => _progress = p);
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDownloading = _progress.status == DownloadStatus.downloading;
    final isPaused = _progress.status == DownloadStatus.paused;
    final isFailed = _progress.status == DownloadStatus.failed;
    final isCompleted = _progress.status == DownloadStatus.completed;
    final pct = (_progress.fraction * 100).toStringAsFixed(0);
    final label =
        '${_progress.mbDownloaded} / ${_progress.mbTotal} MB ($pct%)';

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          children: [
            SizedBox(
              width: 48,
              height: 48,
              child: CircularProgressIndicator(
                value: isCompleted ? 1.0 : _progress.fraction,
                strokeWidth: 5,
                backgroundColor: theme.colorScheme.outlineVariant,
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    _titleForStatus(_progress.status),
                    style: theme.textTheme.titleSmall,
                  ),
                  const SizedBox(height: 4),
                  LinearProgressIndicator(
                    value: isCompleted ? 1.0 : _progress.fraction,
                    backgroundColor: theme.colorScheme.outlineVariant,
                  ),
                  const SizedBox(height: 4),
                  Text(label, style: theme.textTheme.bodySmall),
                  if (_progress.errorMessage != null)
                    Text(
                      _progress.errorMessage!,
                      style: TextStyle(color: theme.colorScheme.error, fontSize: 12),
                    ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          children: [
            if (isDownloading)
              ElevatedButton.icon(
                onPressed: () => ModelDownloadManager().pauseDownload(),
                icon: const Icon(Icons.pause),
                label: const Text('Pause'),
              ),
            if (isPaused)
              ElevatedButton.icon(
                onPressed: () => ModelDownloadManager().resumeDownload(),
                icon: const Icon(Icons.play_arrow),
                label: const Text('Resume'),
              ),
            if (isFailed)
              ElevatedButton.icon(
                onPressed: () => ModelDownloadManager().resumeDownload(),
                icon: const Icon(Icons.refresh),
                label: const Text('Retry'),
              ),
            if (!isCompleted)
              TextButton(
                onPressed: () => ModelDownloadManager().cancelDownload(),
                child: const Text('Cancel'),
              ),
          ],
        ),
      ],
    );
  }

  String _titleForStatus(DownloadStatus s) {
    switch (s) {
      case DownloadStatus.pending:
        return 'Download pending';
      case DownloadStatus.downloading:
        return 'Downloading model';
      case DownloadStatus.paused:
        return 'Download paused';
      case DownloadStatus.completed:
        return 'Download complete';
      case DownloadStatus.failed:
        return 'Download failed';
    }
  }
}
