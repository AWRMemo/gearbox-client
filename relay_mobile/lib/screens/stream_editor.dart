import 'package:flutter/material.dart';
import 'package:share_plus/share_plus.dart';
import '../services/relay_service.dart';
import '../src/rust/api/types.dart';

class StreamEditor extends StatefulWidget {
  const StreamEditor({super.key});

  @override
  State<StreamEditor> createState() => _StreamEditorState();
}

class _StreamEditorState extends State<StreamEditor> {
  final RelayService _relay = RelayService();
  final _titleController = TextEditingController();
  final _descriptionController = TextEditingController();

  List<ListedHighlightResponse> _highlights = [];
  final Set<String> _selectedIds = {};
  bool _isLoading = true;
  bool _isPublishing = false;
  String? _publishedLink;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadHighlights();
  }

  Future<void> _loadHighlights() async {
    setState(() => _isLoading = true);
    try {
      _highlights = await _relay.listHighlights();
    } catch (e) {
      _error = e.toString();
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _publish() async {
    final title = _titleController.text.trim();
    if (title.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter a Stream title')),
      );
      return;
    }
    if (_selectedIds.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Select at least one highlight')),
      );
      return;
    }

    setState(() {
      _isPublishing = true;
      _error = null;
    });

    try {
      final streamId = await _relay.createStream(title, _descriptionController.text.trim());
      for (final highlightId in _selectedIds) {
        await _relay.addHighlightToStream(streamId, highlightId);
      }
      setState(() {
        _publishedLink = 'relay://subscribe/$streamId';
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Stream published!'), duration: Duration(seconds: 2)),
        );
      }
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      setState(() => _isPublishing = false);
    }
  }

  void _share() {
    if (_publishedLink == null) return;
    Share.share(_publishedLink!, subject: _titleController.text);
  }

  @override
  void dispose() {
    _titleController.dispose();
    _descriptionController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('New Stream')),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _highlights.isEmpty
              ? Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.inbox_outlined, size: 64, color: theme.colorScheme.onSurfaceVariant),
                      const SizedBox(height: 8),
                      Text('Capture some highlights first', style: theme.textTheme.bodyLarge),
                    ],
                  ),
                )
              : Column(
                  children: [
                    Expanded(
                      child: SingleChildScrollView(
                        padding: const EdgeInsets.all(16),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Select Highlights', style: theme.textTheme.titleMedium),
                            const SizedBox(height: 8),
                            Text('${_selectedIds.length} selected', style: theme.textTheme.bodySmall),
                            const SizedBox(height: 8),
                            ...(_highlights.map((h) => CheckboxListTile(
                                  title: Text(h.summary, maxLines: 2, overflow: TextOverflow.ellipsis),
                                  subtitle: h.tags.isNotEmpty
                                      ? Wrap(spacing: 4, children: h.tags.map((t) => Chip(
                                            label: Text(t, style: const TextStyle(fontSize: 10)),
                                            visualDensity: VisualDensity.compact,
                                            materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                                          )).toList())
                                      : null,
                                  value: _selectedIds.contains(h.id),
                                  onChanged: (checked) {
                                    setState(() {
                                      if (checked == true) {
                                        _selectedIds.add(h.id);
                                      } else {
                                        _selectedIds.remove(h.id);
                                      }
                                    });
                                  },
                                ))),
                            const SizedBox(height: 16),
                            Text('Stream Details', style: theme.textTheme.titleMedium),
                            const SizedBox(height: 8),
                            TextField(
                              controller: _titleController,
                              decoration: const InputDecoration(
                                labelText: 'Title',
                                hintText: 'My Reading List',
                                border: OutlineInputBorder(),
                              ),
                            ),
                            const SizedBox(height: 12),
                            TextField(
                              controller: _descriptionController,
                              maxLines: 3,
                              decoration: const InputDecoration(
                                labelText: 'Description (optional)',
                                hintText: 'A collection of highlights about...',
                                border: OutlineInputBorder(),
                              ),
                            ),
                            if (_error != null) ...[
                              const SizedBox(height: 12),
                              Card(
                                color: theme.colorScheme.errorContainer,
                                child: Padding(
                                  padding: const EdgeInsets.all(12),
                                  child: Text(_error!, style: TextStyle(color: theme.colorScheme.onErrorContainer)),
                                ),
                              ),
                            ],
                            if (_publishedLink != null) ...[
                              const SizedBox(height: 16),
                              Card(
                                color: theme.colorScheme.primaryContainer,
                                child: Padding(
                                  padding: const EdgeInsets.all(16),
                                  child: Column(
                                    crossAxisAlignment: CrossAxisAlignment.start,
                                    children: [
                                      Text('Published!', style: theme.textTheme.titleMedium?.copyWith(color: theme.colorScheme.onPrimaryContainer)),
                                      const SizedBox(height: 8),
                                      SelectableText(_publishedLink!, style: TextStyle(color: theme.colorScheme.onPrimaryContainer, fontFamily: 'monospace')),
                                      const SizedBox(height: 12),
                                      FilledButton.icon(
                                        onPressed: _share,
                                        icon: const Icon(Icons.share),
                                        label: const Text('Share Stream Link'),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ],
                          ],
                        ),
                      ),
                    ),
                    SafeArea(
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: SizedBox(
                          width: double.infinity,
                          child: FilledButton.icon(
                            onPressed: _isPublishing || _publishedLink != null ? null : _publish,
                            icon: _isPublishing
                                ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                                : const Icon(Icons.publish),
                            label: Text(_isPublishing ? 'Publishing...' : 'Publish Stream'),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
    );
  }
}
