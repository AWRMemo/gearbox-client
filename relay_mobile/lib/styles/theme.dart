import 'package:flutter/material.dart';

class RelayTheme {
  static ThemeData lightTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorSchemeSeed: const Color(0xFF1565c0),
    scaffoldBackgroundColor: const Color(0xFFf5f5f5),
    cardTheme: const CardThemeData(color: Color(0xFFFFFFFF)),
    appBarTheme: const AppBarThemeData(
      backgroundColor: Color(0xFFf5f5f5),
      foregroundColor: Color(0xFF222222),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: const Color(0xFFe3f2fd),
      labelStyle: const TextStyle(color: Color(0xFF1565c0), fontSize: 11),
      visualDensity: VisualDensity.compact,
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      side: BorderSide.none,
    ),
  );

  static ThemeData darkTheme = ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorSchemeSeed: const Color(0xFFa0d8ef),
    scaffoldBackgroundColor: const Color(0xFF1a1a2e),
    cardTheme: const CardThemeData(color: Color(0xFF16213e)),
    appBarTheme: const AppBarThemeData(
      backgroundColor: Color(0xFF1a1a2e),
      foregroundColor: Color(0xFFf0f0f0),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: const Color(0xFF0f3460),
      labelStyle: const TextStyle(color: Color(0xFFa0d8ef), fontSize: 11),
      visualDensity: VisualDensity.compact,
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      side: BorderSide.none,
    ),
  );

  static const String storageKey = 'relay_theme';

  static ThemeMode storedMode(String? pref) {
    if (pref == 'dark') return ThemeMode.dark;
    if (pref == 'light') return ThemeMode.light;
    return ThemeMode.system;
  }
}
