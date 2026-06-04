enum DownloadStatus { pending, downloading, paused, completed, failed }

class DownloadProgress {
  final int bytesDownloaded;
  final int totalBytes;
  final DownloadStatus status;
  final String? errorMessage;

  const DownloadProgress({
    required this.bytesDownloaded,
    required this.totalBytes,
    required this.status,
    this.errorMessage,
  });

  double get fraction =>
      totalBytes > 0 ? (bytesDownloaded / totalBytes).clamp(0.0, 1.0) : 0.0;

  String get mbDownloaded =>
      (bytesDownloaded / 1024 / 1024).toStringAsFixed(1);

  String get mbTotal => (totalBytes / 1024 / 1024).toStringAsFixed(1);
}
