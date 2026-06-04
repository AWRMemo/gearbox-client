import 'package:flutter/material.dart';

class SearchFilterChips extends StatelessWidget {
  final List<String> selectedTags;
  final ValueChanged<List<String>> onTagsChanged;
  final List<String> availableTags;
  final DateTimeRange? dateFrom;
  final DateTimeRange? dateTo;
  final ValueChanged<DateTimeRange?> onDateFromChanged;
  final ValueChanged<DateTimeRange?> onDateToChanged;
  final bool semanticEnabled;
  final ValueChanged<bool> onSemanticChanged;

  const SearchFilterChips({
    super.key,
    required this.selectedTags,
    required this.onTagsChanged,
    this.availableTags = const [],
    this.dateFrom,
    this.dateTo,
    required this.onDateFromChanged,
    required this.onDateToChanged,
    required this.semanticEnabled,
    required this.onSemanticChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Row(children: [
        const Text('Semantic search', style: TextStyle(fontSize: 13)),
        const SizedBox(width: 8),
        Switch(value: semanticEnabled, onChanged: onSemanticChanged, materialTapTargetSize: MaterialTapTargetSize.shrinkWrap),
      ]),
      if (availableTags.isNotEmpty) ...[
        const SizedBox(height: 8),
        const Text('Filter by tag', style: TextStyle(fontSize: 13)),
        const SizedBox(height: 4),
        Wrap(spacing: 6, runSpacing: 4, children: availableTags.map((tag) {
          final selected = selectedTags.contains(tag);
          return FilterChip(
            label: Text(tag, style: const TextStyle(fontSize: 12)),
            selected: selected,
            onSelected: (v) {
              if (v) {
                onTagsChanged([...selectedTags, tag]);
              } else {
                onTagsChanged(selectedTags.where((t) => t != tag).toList());
              }
            },
            visualDensity: VisualDensity.compact,
          );
        }).toList()),
      ],
      const SizedBox(height: 8),
      Row(children: [
        TextButton.icon(
          icon: const Icon(Icons.calendar_today, size: 16),
          label: const Text('Date from', style: TextStyle(fontSize: 12)),
          onPressed: () async {
            final picked = await showDateRangePicker(context: context, firstDate: DateTime(2020), lastDate: DateTime.now());
            if (picked != null) onDateFromChanged(picked);
          },
        ),
        const SizedBox(width: 8),
        TextButton.icon(
          icon: const Icon(Icons.calendar_today, size: 16),
          label: const Text('Date to', style: TextStyle(fontSize: 12)),
          onPressed: () async {
            final picked = await showDateRangePicker(context: context, firstDate: DateTime(2020), lastDate: DateTime.now());
            if (picked != null) onDateToChanged(picked);
          },
        ),
      ]),
    ]);
  }
}
