import 'package:flutter/material.dart';
import 'package:share_plus/share_plus.dart';
import '../services/relay_service.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  final RelayService _relay = RelayService();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  bool _isSyncing = false;
  bool _isLoggedIn = false;
  String? _email;
  int _pendingCount = 0;
  String _syncStatus = 'idle';
  bool _telemetryOptOut = true;

  @override
  void initState() {
    super.initState();
    _refreshStatus();
  }

  Future<void> _refreshStatus() async {
    try {
      final auth = await _relay.getAuthStatus();
      final sync = await _relay.getSyncStatus();
      final telemetryOptOut = await _relay.getTelemetryOptOut();
      setState(() {
        _isLoggedIn = auth.loggedIn;
        _email = auth.email;
        _syncStatus = sync.status;
        _pendingCount = sync.pendingCount.toInt();
        _telemetryOptOut = telemetryOptOut;
      });
    } catch (_) {}
  }

  Future<void> _syncNow() async {
    setState(() => _isSyncing = true);
    try {
      await _relay.syncNow();
      await _refreshStatus();
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Sync complete'), duration: Duration(seconds: 2)));
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Sync failed: $e')));
    } finally {
      setState(() => _isSyncing = false);
    }
  }

  Future<void> _exportData() async {
    try {
      final path = await _relay.exportData();
      if (mounted) await Share.shareXFiles([XFile(path)], text: 'Relay Export');
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Export failed: $e')));
    }
  }

  Future<void> _clearData() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (BuildContext ctx) => AlertDialog(
        title: const Text('Clear All Data'),
        content: const Text('This permanently deletes all highlights, streams, and settings. This cannot be undone.'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.pop(ctx, true),
            style: FilledButton.styleFrom(backgroundColor: Theme.of(context).colorScheme.error),
            child: const Text('Delete Everything')),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await _relay.clearData();
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('All data cleared')));
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Clear failed: $e')));
    }
  }

  Future<void> _logIn() async {
    try {
      await _relay.logIn(_emailController.text.trim(), _passwordController.text);
      _emailController.clear(); _passwordController.clear();
      await _refreshStatus();
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Login failed: $e')));
    }
  }

  Future<void> _createAccount() async {
    try {
      await _relay.createAccount(_emailController.text.trim(), _passwordController.text);
      _emailController.clear(); _passwordController.clear();
      await _refreshStatus();
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Account creation failed: $e')));
    }
  }

  Future<void> _logOut() async {
    await _relay.logOut();
    await _refreshStatus();
  }

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(child: Padding(padding: const EdgeInsets.all(16), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Account', style: theme.textTheme.titleMedium), const SizedBox(height: 12),
            if (_isLoggedIn) ...[
              ListTile(leading: CircleAvatar(child: Text((_email ?? '?').substring(0, 1).toUpperCase())), title: Text(_email ?? ''),
                trailing: TextButton(onPressed: _logOut, child: const Text('Log Out'))),
            ] else ...[
              TextField(controller: _emailController, decoration: const InputDecoration(labelText: 'Email', border: OutlineInputBorder()), keyboardType: TextInputType.emailAddress),
              const SizedBox(height: 12),
              TextField(controller: _passwordController, decoration: const InputDecoration(labelText: 'Password', border: OutlineInputBorder()), obscureText: true),
              const SizedBox(height: 12),
              Row(children: [Expanded(child: FilledButton(onPressed: _logIn, child: const Text('Log In'))), const SizedBox(width: 12), Expanded(child: OutlinedButton(onPressed: _createAccount, child: const Text('Create Account')))]),
            ],
          ]))),
          const SizedBox(height: 16),
          Card(child: Padding(padding: const EdgeInsets.all(16), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Sync', style: theme.textTheme.titleMedium), const SizedBox(height: 8),
            ListTile(
              leading: Icon(_syncStatus == 'syncing' ? Icons.sync : _syncStatus == 'synced' ? Icons.cloud_done_outlined : Icons.cloud_off_outlined),
              title: Text(_syncStatus == 'syncing' ? 'Syncing...' : 'Status: $_syncStatus'),
              subtitle: Text('$_pendingCount pending'),
              trailing: FilledButton.tonalIcon(
                onPressed: _isSyncing ? null : _syncNow,
                icon: _isSyncing ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.sync),
                label: const Text('Sync Now')),
            ),
          ]))),
          const SizedBox(height: 16),
          Card(child: Padding(padding: const EdgeInsets.all(16), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Data', style: theme.textTheme.titleMedium), const SizedBox(height: 8),
            ListTile(leading: const Icon(Icons.file_download_outlined), title: const Text('Export Data'), subtitle: const Text('Save all highlights as ZIP'), trailing: const Icon(Icons.chevron_right), onTap: _exportData),
            ListTile(leading: Icon(Icons.delete_forever_outlined, color: theme.colorScheme.error), title: Text('Clear All Data', style: TextStyle(color: theme.colorScheme.error)), subtitle: const Text('Irreversible'), trailing: const Icon(Icons.chevron_right), onTap: _clearData),
          ]))),
          const SizedBox(height: 16),
          Card(child: Padding(padding: const EdgeInsets.all(16), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Privacy', style: theme.textTheme.titleMedium), const SizedBox(height: 8),
            ListTile(
              leading: const Icon(Icons.analytics_outlined),
              title: const Text('Telemetry'),
              subtitle: const Text('Anonymous crash reporting and performance metrics'),
              trailing: Switch(
                value: !_telemetryOptOut,
                onChanged: (bool v) async {
                  setState(() => _telemetryOptOut = !v);
                  try {
                    await _relay.setTelemetryOptOut(!v);
                  } catch (_) {}
                },
              ),
            ),
          ]))),
          const SizedBox(height: 16),
          Card(child: Padding(padding: const EdgeInsets.all(16), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('About', style: theme.textTheme.titleMedium), const SizedBox(height: 8),
            const ListTile(leading: Icon(Icons.info_outline), title: Text('Gearbox Relay'), subtitle: Text('v0.1.0 — Local-first AI knowledge pipeline')),
          ]))),
        ],
      ),
    );
  }
}
