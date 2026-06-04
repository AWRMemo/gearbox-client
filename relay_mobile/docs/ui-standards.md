# Gearbox Relay Mobile UI Standards

## Naming Conventions

All screen files must use the `{feature}_screen.dart` pattern:
- `capture_screen.dart`
- `history_screen.dart`
- `search_screen.dart`
- `settings_screen.dart`
- `stream_editor.dart`

Service files use the `{platform}_ai_service.dart` or `{feature}_service.dart` pattern.

## Minimum Tap Target

All interactive elements must have a minimum effective tap area of **48x48dp** per Material Design guidelines. For `Checkbox`, `IconButton`, and `GestureDetector`, wrap with `InkWell` or `SizedBox` if the visual element is smaller.

## `const` Constructors

Use `const` constructors for all stateless widgets, static layouts, and unchanging widget trees wherever possible. This reduces rebuilds and improves performance.

## Lifecycle Cleanup

Any widget that creates a `TextEditingController`, `ScrollController`, `AnimationController`, `Timer`, `StreamSubscription`, or `FocusNode` **must** call `.dispose()` in the widget's `dispose()` method.

## State Management

For Sprint 12 and v1, use only:
- `StatefulWidget` + `setState` for local UI state
- `ValueNotifier` / `ChangeNotifier` for subtree-wide state
- No global store (no Riverpod, Bloc, or Redux) for v1 without explicit approval.

## Accessibility

- Every `TextFormField` and `TextField` must have a `labelText` or `decoration.label`.
- Every icon-only button must have a `tooltip` and `semanticLabel`.
- All `ListView.builder` items should use a `Key` (e.g. `ValueKey(item.id)`) for stable reordering and screen-reader focus.

## Error Handling in UI

- Never show raw stack traces or `Exception.toString()` to users.
- Use the `toast` system for transient errors (`SnackBar` on mobile, toast on desktop).
- Show inline `Text("Error: ...")` only for persistent form validation errors.

## Platform Checks

When branching on platform for AI or native features, always check `!kIsWeb` first:

```dart
if (Platform.isAndroid && !kIsWeb) { ... }
if (Platform.isIOS && !kIsWeb) { ... }
```

## Asset Management

- Model files (`.task`, `.gguf`, `.bin`) must **never** be committed to Git.
- Add a `.gitignore` rule: `assets/models/*.task`, `assets/models/*.bin`.
- Provide a `README.md` in `assets/models/` with download/conversion instructions.

## CI Compliance

- `flutter analyze --fatal-infos` must pass before merge.
- No warnings from `flutter analyze` (or explicitly suppressed with a justification comment).
