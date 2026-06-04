import 'package:flutter/material.dart';
import '../services/relay_service.dart';
import '../src/rust/api/types.dart';

class SubscribeScreen extends StatefulWidget {
  final String streamId;
  const SubscribeScreen({super.key, required this.streamId});

  @override
  State<SubscribeScreen> createState() => _SubscribeScreenState();
}

class _SubscribeScreenState extends State<SubscribeScreen> {
  final RelayService _relay = RelayService();
  bool _isLoading = true;
  StreamInfoResponse? _metadata;
  String? _error;
  bool _isSubscribing = false;

  @override
  void initState() {
    super.initState();
    _fetchMetadata();
  }

  Future<void> _fetchMetadata() async {
    setState(() => _isLoading = true);
    try {
      final meta = await _relay.getStream(widget.streamId);
      setState(() => _metadata = meta);
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _subscribe() async {
    setState(() => _isSubscribing = true);
    try {
      await _relay.subscribeToStream(widget.streamId);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Subscribed! Highlights imported.')));
        Navigator.of(context).pop();
      }
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Subscribe failed: $e')));
    } finally {
      setState(() => _isSubscribing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('Subscribe to Stream')),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? Center(child: Column(mainAxisSize: MainAxisSize.min, children: [
                  Icon(Icons.error_outline, size: 48, color: theme.colorScheme.error), const SizedBox(height: 12), Text(_error!),
                  const SizedBox(height: 12), FilledButton(onPressed: _fetchMetadata, child: const Text('Retry')),
                ]))
              : Padding(
                  padding: const EdgeInsets.all(24.0),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Card(child: Padding(padding: const EdgeInsets.all(20), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                      Text(_metadata?.title ?? 'Untitled Stream', style: theme.textTheme.headlineSmall), const SizedBox(height: 8),
                      Text(_metadata?.description ?? '', style: theme.textTheme.bodyLarge?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
                      const SizedBox(height: 16),
                      Row(children: [Icon(Icons.person_outline, size: 18, color: theme.colorScheme.primary), const SizedBox(width: 6),
                        Text('Curated by ${_metadata?.userId ?? 'Relay User'}', style: theme.textTheme.bodyMedium)]),
                    ]))),
                    const SizedBox(height: 24),
                    Text('When you subscribe, all highlights from this Stream will be imported into your local knowledge base.', style: theme.textTheme.bodyMedium?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
                    const Spacer(),
                    SizedBox(width: double.infinity, child: FilledButton.icon(
                      onPressed: _isSubscribing ? null : _subscribe,
                      icon: _isSubscribing ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white)) : const Icon(Icons.rss_feed),
                      label: Text(_isSubscribing ? 'Subscribing...' : 'Subscribe'))),
                  ])),
    );
  }
}
