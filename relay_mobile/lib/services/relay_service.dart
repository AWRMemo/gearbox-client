import 'package:path_provider/path_provider.dart';
import '../src/rust/api/relay_api.dart' as api;
import '../src/rust/frb_generated.dart';
import '../src/rust/api/types.dart';
class RelayService {
  static final RelayService _instance = RelayService._();
  factory RelayService() => _instance;
  RelayService._();

  bool _initialized = false;

  Future<void> init() async {
    if (_initialized) return;
    await RustLib.init();
    final dir = await getApplicationDocumentsDirectory();
    final dataDir = '${dir.path}/relay_data';
    api.initCore(dataDir: dataDir);
    _initialized = true;
  }

  Future<api.EnrichResult> enrichAndStore(String text) async {
    await init();
    return api.enrichAndStore(text: text);
  }

  Future<String> storeHighlight({
    required String text,
    required String summary,
    required List<String> tags,
    String? sourceUrl,
    String? sourceTitle,
    String? sourceAuthor,
  }) async {
    await init();
    return api.storeHighlight(
      text: text,
      summary: summary,
      tags: tags,
      sourceUrl: sourceUrl,
      sourceTitle: sourceTitle,
      sourceAuthor: sourceAuthor,
    );
  }

  Future<List<SearchResultResponse>> searchHighlights(String query, {int limit = 20}) async {
    await init();
    return api.searchHighlights(query: query, limit: limit);
  }

  Future<List<ListedHighlightResponse>> listHighlights({int limit = 50, int offset = 0}) async {
    await init();
    return api.listStoredHighlights(limit: limit, offset: offset);
  }

  Future<void> deleteHighlight(String id) async {
    await init();
    return api.deleteHighlight(id: id);
  }

  Future<SyncReportResponse> syncNow() async {
    await init();
    return api.syncNow();
  }

  Future<SyncStatusResponse> getSyncStatus() async {
    await init();
    return api.getSyncStatus();
  }

  Future<AuthStatusResponse> getAuthStatus() async {
    await init();
    return api.getAuthStatus();
  }

  Future<AuthResultResponse> createAccount(String email, String password) async {
    await init();
    return api.createAccount(email: email, password: password);
  }

  Future<AuthResultResponse> logIn(String email, String password) async {
    await init();
    return api.logIn(email: email, password: password);
  }

  Future<void> logOut() async {
    await init();
    return api.logOut();
  }

  Future<StreamInfoResponse> getStream(String streamId) async {
    await init();
    return api.getStream(streamId: streamId);
  }

  Future<void> subscribeToStream(String streamId) async {
    await init();
    return api.subscribeToStream(streamId: streamId);
  }

  Future<String> createStream(String title, String description) async {
    await init();
    return api.createStream(title: title, description: description);
  }

  Future<void> addHighlightToStream(String streamId, String highlightId) async {
    await init();
    return api.addHighlightToStream(streamId: streamId, highlightId: highlightId);
  }

  Future<void> registerDeviceToken(String token, {String platform = 'android'}) async {
    await init();
    return api.registerDeviceToken(token: token, platform: platform);
  }

  Future<List<StreamInfoResponse>> listMyStreams() async {
    await init();
    return api.listMyStreams();
  }

  Future<String> exportData() async {
    await init();
    return api.exportLocalData();
  }

  Future<void> clearData() async {
    await init();
    return api.clearLocalData();
  }

  Future<bool> getTelemetryOptOut() async {
    await init();
    return api.getTelemetryOptOut();
  }

  Future<void> setTelemetryOptOut(bool optOut) async {
    await init();
    return api.setTelemetryOptOut(optOut: optOut);
  }
}
