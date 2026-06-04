import 'package:flutter/material.dart';
import '../services/relay_service.dart';
import '../src/rust/api/types.dart';

class HistoryScreen extends StatefulWidget {
  const HistoryScreen({super.key});

  @override
  State<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends State<HistoryScreen> {
  final RelayService _relay = RelayService();
  final List<ListedHighlightResponse> _entries = [];
  bool _isLoading = false;
  bool _hasMore = true;
  int _offset = 0;
  static const int _limit = 20;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadMore();
  }

  Future<void> _loadMore() async {
    if (_isLoading || !_hasMore) return;
    setState(() => _isLoading = true);
    try {
      final results = await _relay.listHighlights(limit: _limit, offset: _offset);
      setState(() {
        _entries.addAll(results);
        _offset += results.length;
        _hasMore = results.length == _limit;
        _error = null;
      });
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _refresh() async {
    setState(() {
      _entries.clear();
      _offset = 0;
      _hasMore = true;
      _error = null;
    });
    await _loadMore();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('History')),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: _buildBody(theme),
      ),
    );
  }

  Widget _buildBody(ThemeData theme) {
    if (_entries.isEmpty && _isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_entries.isEmpty && _error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.error_outline, color: theme.colorScheme.error, size: 48),
              const SizedBox(height: 12),
              Text(_error!, textAlign: TextAlign.center),
              const SizedBox(height: 12),
              FilledButton(onPressed: _refresh, child: const Text('Retry')),
            ],
          ),
        ),
      );
    }
    if (_entries.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.inbox_outlined, color: theme.colorScheme.outline, size: 48),
            const SizedBox(height: 12),
            Text('No highlights yet', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            Text('Copy text to get started.', style: theme.textTheme.bodyMedium),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: _entries.length + (_hasMore ? 1 : 0),
      itemBuilder: (BuildContext context, int index) {
        if (index == _entries.length) {
          return Padding(
            padding: const EdgeInsets.all(16),
            child: Center(
              child: _isLoading
                  ? const CircularProgressIndicator()
                  : FilledButton(onPressed: _loadMore, child: const Text('Load More')),
            ),
          );
        }
        final entry = _entries[index];
        return _HighlightCard(entry: entry, theme: theme);
      },
    );
  }
}

class _HighlightCard extends StatelessWidget {
  final ListedHighlightResponse entry;
  final ThemeData theme;

  const _HighlightCard({super.key, required this.entry, required this.theme});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              entry.summary.isNotEmpty ? entry.summary : entry.text,
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
              style: theme.textTheme.bodyLarge,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 6,
              runSpacing: 4,
              children: entry.tags
                   .map((String tag) => Chip(
                        label: Text(tag),
                        visualDensity: VisualDensity.compact,
                        padding: EdgeInsets.zero,
                        labelStyle: theme.textTheme.labelSmall,
                      ))
                  .toList(),
            ),
            if (entry.sourceUrl != null) ...[
              const SizedBox(height: 6),
              Text(
                entry.sourceUrl!,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.primary,
                    ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
