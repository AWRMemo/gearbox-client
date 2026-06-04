import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../services/model_download_manager.dart';
import '../services/download_progress.dart';
import '../widgets/model_download_progress.dart';

class OnboardingScreen extends StatefulWidget {
  final VoidCallback onDone;
  const OnboardingScreen({super.key, required this.onDone});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final PageController _pageController = PageController();
  int _currentPage = 0;
  bool _modelReady = false;
  bool _skipped = false;

  final List<_OnboardingPage> _basePages = const [
    _OnboardingPage(
      title: 'Welcome to Gearbox Relay',
      body: 'Your personal knowledge base, powered entirely by on-device AI. Your highlights never leave your device unless you choose to sync.',
    ),
    _OnboardingPage(
      title: 'Capture Anything',
      body: 'Copy any text to instantly capture it. Relay enriches it with a smart summary and tags using a local AI model — no cloud, no tracking.',
    ),
    _OnboardingPage(
      title: 'Curate Streams',
      body: 'Build Streams to organise and share highlight collections. Publish them with a single click. Anyone with the link can subscribe.',
    ),
    _OnboardingPage(
      title: 'Sync & Privacy',
      body: 'Sign in to sync across devices. All data is encrypted end-to-end. We can\'t read your highlights, and neither can anyone else.',
    ),
  ];

  Future<void> _finish() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool('relay_onboarding_seen', true);
    widget.onDone();
  }

  void _onModelReady() {
    if (mounted) setState(() => _modelReady = true);
  }

  void _onSkipModel() {
    if (mounted) setState(() => _skipped = true);
    _pageController.nextPage(
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeInOut,
    );
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final pages = _buildPages();
    final isLast = _currentPage == pages.length - 1;
    final canAdvance = (isLast) || (_currentPage != 2) || _modelReady || _skipped;

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            children: [
              Expanded(
                child: PageView.builder(
                  controller: _pageController,
                  physics: const NeverScrollableScrollPhysics(),
                  onPageChanged: (int i) => setState(() => _currentPage = i),
                  itemCount: pages.length,
                  itemBuilder: (BuildContext context, int i) =>
                      _PageContent(page: pages[i]),
                ),
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(pages.length, (int i) => Container(
                  margin: const EdgeInsets.symmetric(horizontal: 4),
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: i == _currentPage
                        ? theme.colorScheme.primary
                        : theme.colorScheme.outlineVariant,
                  ),
                )),
              ),
              const SizedBox(height: 24),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  TextButton(
                    onPressed: _finish,
                    child: const Text('Skip'),
                  ),
                  FilledButton(
                    onPressed: canAdvance
                        ? () {
                            if (isLast) {
                              _finish();
                            } else {
                              _pageController.nextPage(
                                duration: const Duration(milliseconds: 300),
                                curve: Curves.easeInOut,
                              );
                            }
                          }
                        : null,
                    child: Text(isLast ? 'Get Started' : 'Next'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<_OnboardingPage> _buildPages() {
    return [
      ..._basePages.take(2),
      const _OnboardingPage(
        title: 'Download your AI model',
        body: 'Relay needs a local AI model (~500 MB) to enrich your highlights. You can download it now or skip and use a lightweight fallback.',
        showDownloadWidget: true,
      ),
      ..._basePages.skip(2),
    ];
  }
}

class _OnboardingPage {
  final String title;
  final String body;
  final bool showDownloadWidget;

  const _OnboardingPage({
    required this.title,
    required this.body,
    this.showDownloadWidget = false,
  });
}

class _PageContent extends StatefulWidget {
  final _OnboardingPage page;
  const _PageContent({super.key, required this.page});

  @override
  State<_PageContent> createState() => _PageContentState();
}

class _PageContentState extends State<_PageContent> {
  DownloadProgress _progress = const DownloadProgress(
    bytesDownloaded: 0,
    totalBytes: 0,
    status: DownloadStatus.pending,
  );

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Icon(Icons.lightbulb_outline, size: 64, color: theme.colorScheme.primary),
        const SizedBox(height: 24),
        Text(widget.page.title,
            style: theme.textTheme.headlineSmall, textAlign: TextAlign.center),
        const SizedBox(height: 16),
        Text(widget.page.body,
            style: theme.textTheme.bodyLarge, textAlign: TextAlign.center),
        if (widget.page.showDownloadWidget) ...[
          const SizedBox(height: 32),
          SizedBox(
            width: double.infinity,
            child: ModelDownloadProgress(
              statusStream: ModelDownloadManager().statusStream,
            ),
          ),
          const SizedBox(height: 12),
          TextButton(
            onPressed: () {
              // skip model download
            },
            child: const Text('Use fallback (skip download)'),
          ),
        ],
      ],
    );
  }
}
