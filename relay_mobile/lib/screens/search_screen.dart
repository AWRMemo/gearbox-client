import 'dart:async';
import 'package:flutter/material.dart';
import '../services/relay_service.dart';
import '../widgets/search_filter_chips.dart';
import '../src/rust/api/types.dart';

class SearchScreen extends StatefulWidget {
  const SearchScreen({super.key});

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final TextEditingController _searchController = TextEditingController();
  final RelayService _relay = RelayService();
  List<ListedHighlightResponse> _highlights = [];
  List<SearchResultResponse> _searchResults = [];
  bool _isSearching = false;
  bool _isLoading = false;
  Timer? _debounce;
  bool _semantic = false;
  List<String> _selectedTags = [];
  DateTimeRange? _dateFrom;
  DateTimeRange? _dateTo;

  @override
  void initState() {
    super.initState();
    _loadAll();
  }

  void _onSearchChanged(String query) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 300), () {
      if (query.trim().isEmpty) {
        _loadAll();
      } else {
        _search(query);
      }
    });
  }

  Future<void> _loadAll() async {
    setState(() { _isLoading = true; _isSearching = false; });
    try {
      _highlights = await _relay.listHighlights();
    } catch (_) {
      _highlights = [];
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _search(String query) async {
    setState(() { _isLoading = true; _isSearching = true; });
    try {
      _searchResults = await _relay.searchHighlights(query);
    } catch (_) {
      _searchResults = [];
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Set<String> get _availableTags {
    final tags = <String>{};
    for (final h in _highlights) {
      tags.addAll(h.tags);
    }
    for (final r in _searchResults) {
      tags.addAll(r.tags);
    }
    return tags;
  }

  Widget _confidenceBadge(double score) {
    final label = score >= 0.8 ? 'high' : score >= 0.5 ? 'medium' : 'low';
    final colors = {
      'high': const Color(0xFF1b5e20),
      'medium': const Color(0xFF5c3a00),
      'low': const Color(0xFF5c1a1a),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(color: colors[label], borderRadius: BorderRadius.circular(3)),
      child: Text(label, style: const TextStyle(fontSize: 10, fontWeight: FontWeight.w600, color: Colors.white)),
    );
  }

  Widget _buildItemCard(String id, String summary, List<String> tags, {double? score, String? text, String? sourceUrl}) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        title: Text(summary, maxLines: 2, overflow: TextOverflow.ellipsis),
        subtitle: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          if (tags.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Wrap(spacing: 4, children: tags.map((t) => Chip(
                label: Text(t, style: const TextStyle(fontSize: 11)),
                visualDensity: VisualDensity.compact,
                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
              )).toList()),
            ),
          if (score != null) ...[
            const SizedBox(height: 4),
            _confidenceBadge(score),
          ],
          if (text != null)
            Text(text, maxLines: 1, overflow: TextOverflow.ellipsis,
              style: TextStyle(fontSize: 11, color: Theme.of(context).colorScheme.onSurfaceVariant)),
        ]),
        trailing: IconButton(
          icon: const Icon(Icons.delete_outline),
          onPressed: () => _deleteHighlight(id),
        ),
      ),
    );
  }

  Future<void> _deleteHighlight(String id) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete Highlight'),
        content: const Text('Permanently remove this highlight?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.pop(ctx, true), child: const Text('Delete')),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await _relay.deleteHighlight(id);
      _loadAll();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Delete failed: $e')));
      }
    }
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('Search')),
      body: Column(children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
          child: SearchBar(
            controller: _searchController,
            onChanged: _onSearchChanged,
            hintText: 'Search highlights...',
            leading: const Icon(Icons.search),
            trailing: [
              if (_searchController.text.isNotEmpty)
                IconButton(icon: const Icon(Icons.clear), onPressed: () {
                  _searchController.clear();
                  _loadAll();
                }),
            ],
          ),
        ),
        if (_isSearching || _highlights.isNotEmpty || _searchResults.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 0),
            child: SearchFilterChips(
              selectedTags: _selectedTags,
              onTagsChanged: (t) => setState(() => _selectedTags = t),
              availableTags: _availableTags.toList(),
              semanticEnabled: _semantic,
              onSemanticChanged: (v) => setState(() => _semantic = v),
              dateFrom: _dateFrom,
              dateTo: _dateTo,
              onDateFromChanged: (d) => setState(() => _dateFrom = d),
              onDateToChanged: (d) => setState(() => _dateTo = d),
            ),
          ),
        Expanded(child: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _isSearching
            ? _searchResults.isEmpty
              ? Center(child: Column(mainAxisSize: MainAxisSize.min, children: [
                  Icon(Icons.inbox_outlined, size: 64, color: theme.colorScheme.onSurfaceVariant),
                  const SizedBox(height: 8),
                  Text('No matches found', style: theme.textTheme.bodyLarge),
                  const SizedBox(height: 4),
                  Text('Try different keywords or adjust filters.', style: TextStyle(fontSize: 13, color: theme.colorScheme.onSurfaceVariant)),
                ]))
              : Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 8, 16, 0),
                    child: Text('${_searchResults.length} result${_searchResults.length != 1 ? 's' : ''}',
                      style: TextStyle(fontSize: 12, color: theme.colorScheme.onSurfaceVariant)),
                  ),
                  Expanded(child: ListView.builder(
                    itemCount: _searchResults.length,
                    itemBuilder: (ctx, i) {
                      final r = _searchResults[i];
                      return _buildItemCard(r.id, r.summary, r.tags, score: r.score, text: r.text);
                    },
                  )),
                ])
            : _highlights.isEmpty
              ? Center(child: Column(mainAxisSize: MainAxisSize.min, children: [
                  Icon(Icons.inbox_outlined, size: 64, color: theme.colorScheme.onSurfaceVariant),
                  const SizedBox(height: 8),
                  Text('No highlights yet', style: theme.textTheme.bodyLarge),
                ]))
              : ListView.builder(
                  itemCount: _highlights.length,
                  itemBuilder: (ctx, i) {
                    final h = _highlights[i];
                    return _buildItemCard(h.id, h.summary, h.tags, sourceUrl: h.sourceUrl);
                  }),
        ),
      ]),
    );
  }
}
