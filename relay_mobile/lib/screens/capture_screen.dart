import 'dart:async';
import 'package:flutter/material.dart';
import '../models/enrichment_output.dart';
import '../services/ai_service.dart';
import '../src/rust/api/relay_api.dart';

class CaptureScreen extends StatefulWidget {
  static String? pendingSharedText;

  const CaptureScreen({super.key});

  @override
  State<CaptureScreen> createState() => _CaptureScreenState();
}

class _CaptureScreenState extends State<CaptureScreen> {
  final TextEditingController _controller = TextEditingController();
  final AiService _aiService = AiService();
  bool _isLoading = false;
  EnrichmentOutput? _result;
  String? _error;
  StreamSubscription<DownloadProgress>? _downloadSub;
  DownloadProgress? _downloadProgress;
  ModelStatus _modelStatus = ModelStatus.ready;

  @override
  void initState() {
    super.initState();
    _checkModelStatus();
    _applyPendingShare();
  }

  void _applyPendingShare() {
    final text = CaptureScreen.pendingSharedText;
    if (text != null && text.isNotEmpty) {
      CaptureScreen.pendingSharedText = null;
      _controller.text = text;
      _controller.selection = TextSelection.collapsed(offset: text.length);
    }
  }

  Future<void> _checkModelStatus() async {
    final status = await _aiService.getModelStatus();
    setState(() => _modelStatus = status);
    if (status == ModelStatus.notDownloaded) {
      _listenToDownloadProgress();
    }
  }

  void _listenToDownloadProgress() {
    final stream = _aiService.modelDownloadProgress;
    if (stream != null) {
      _downloadSub = stream.listen((progress) {
        setState(() => _downloadProgress = progress);
        if (progress.status == 'completed') {
          setState(() => _modelStatus = ModelStatus.ready);
        } else if (progress.status == 'failed') {
          setState(() => _modelStatus = ModelStatus.error);
        }
      });
    }
  }

  Future<void> _capture() async {
    final text = _controller.text.trim();
    if (text.isEmpty) return;

    setState(() {
      _isLoading = true;
      _error = null;
      _result = null;
    });

    try {
      final output = await _aiService.enrichHighlight(text);
      setState(() => _result = output);
    } catch (e, stack) {
      if (mounted) {
        setState(() => _error = 'Enrichment failed: $e');
      }
      debugPrintStack(stackTrace: stack, label: 'CaptureScreen._capture');
    } finally {
      if (mounted) {
        setState(() => _isLoading = false);
      }
    }
  }

  void _retry() => setState(() {
    _error = null;
    _result = null;
  });

  @override
  void dispose() {
    _controller.dispose();
    _downloadSub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final bool canCapture = _modelStatus == ModelStatus.ready && !_isLoading;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Capture'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: _controller,
              maxLines: 6,
              decoration: InputDecoration(
                hintText: 'Paste text here...',
                border: OutlineInputBorder(borderRadius: BorderRadius.circular(12)),
                filled: true,
                fillColor: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
              ),
            ),
            const SizedBox(height: 16),

            // Model readiness / download state
            if (_modelStatus == ModelStatus.notDownloaded && _downloadProgress != null)
              _buildDownloadCard(theme)
            else if (_modelStatus == ModelStatus.error)
              _buildErrorCard(theme, 'AI model download failed. Please restart the app.')

            const SizedBox(height: 8),
            FilledButton.icon(
              onPressed: canCapture ? _capture : null,
              icon: _isLoading
                  ? const SizedBox(
                      width: 16, height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                    )
                  : const Icon(Icons.auto_awesome),
              label: Text(_isLoading ? 'Analyzing with local AI...' : 'Capture & Enrich'),
            ),

            if (_error != null) ...[
              const SizedBox(height: 16),
              Card(
                color: theme.colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(_error!, style: TextStyle(color: theme.colorScheme.onErrorContainer)),
                      const SizedBox(height: 8),
                      TextButton(onPressed: _retry, child: const Text('Retry')),
                    ],
                  ),
                ),
              ),
            ],

            if (_result != null) ...[
              const SizedBox(height: 16),
              _buildResultCard(theme, _result!),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildDownloadCard(ThemeData theme) {
    final p = _downloadProgress!;
    final percent = p.totalBytes > 0 ? p.bytesDownloaded / p.totalBytes : 0.0;
    final mbDownloaded = (p.bytesDownloaded / 1024 / 1024).toStringAsFixed(1);
    final mbTotal = (p.totalBytes / 1024 / 1024).toStringAsFixed(1);

    return Card(
      color: theme.colorScheme.primaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Downloading AI model...',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
            const SizedBox(height: 8),
            LinearProgressIndicator(value: percent > 0 ? percent : null),
            const SizedBox(height: 8),
            Text(
              '$mbDownloaded / $mbTotal MB',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildErrorCard(ThemeData theme, String message) {
    return Card(
      color: theme.colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Text(
          message,
          style: TextStyle(color: theme.colorScheme.onErrorContainer),
        ),
      ),
    );
  }

  Widget _buildResultCard(ThemeData theme, EnrichmentOutput result) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Summary', style: theme.textTheme.titleMedium),
            const SizedBox(height: 4),
            Text(result.summary, style: theme.textTheme.bodyLarge),
            const SizedBox(height: 12),
            Text('Tags', style: theme.textTheme.titleMedium),
            const SizedBox(height: 4),
            Wrap(
              spacing: 6,
              runSpacing: 4,
              children: result.tags
                  .map((String tag) => Chip(label: Text(tag), visualDensity: VisualDensity.compact))
                  .toList(),
            ),
            if (result.connectionSuggestion != null) ...[
              const SizedBox(height: 12),
              Text('Connection', style: theme.textTheme.titleMedium),
              const SizedBox(height: 4),
              Text(
                result.connectionSuggestion!.bridgingSentence,
                style: theme.textTheme.bodyMedium?.copyWith(color: theme.colorScheme.primary),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
