import { useState, useEffect } from 'react'
import { getStoredTheme, setStoredTheme, applyTheme } from './styles/theme'
import {
  useCapture,
  useFeed,
  useSearch,
  useSettings,
  useStreams,
  useStreamViewer,
  useModelStatus,
  useHistory,
  useToast,
} from './hooks'
import {
  TabBar,
  CapturePanel,
  SearchPanel,
  FeedPanel,
  StreamsPanel,
  StreamDetailPanel,
  StreamViewerPanel,
  SettingsPanel,
  HistoryPanel,
  ToastContainer,
  OnboardingModal,
  FollowingFeed,
  ReviewPanel,
} from './components'
import { onOpenUrl, getCurrent } from './lib/deeplink'
import { registerKeyboardShortcuts } from './lib/keyboard'
import { listen } from '@tauri-apps/api/event'
import type { Tab, StreamInfo } from './types'
import './index.css'

export default function App() {
  const [tab, setTab] = useState<Tab>('capture')
  const [theme, setTheme] = useState(() => {
    const stored = getStoredTheme()
    applyTheme(stored)
    return stored
  })

  const handleThemeChange = (mode: 'light' | 'dark') => {
    setTheme(mode)
    setStoredTheme(mode)
    applyTheme(mode)
  }

  const capture = useCapture()
  const search = useSearch()
  const streams = useStreams()
  const feed = useFeed()
  const settings = useSettings()
  const viewer = useStreamViewer()
  const model = useModelStatus()
  const history = useHistory()
  const { toast } = useToast()

  useEffect(() => {
    streams.loadStreams()
    settings.loadProfile()

    registerKeyboardShortcuts(() => {
      setTab('capture')
      capture.handleCapture()
    })

    async function initDeepLink() {
      try {
        await onOpenUrl((urls: string[]) => {
          for (const payload of urls) {
            const match = payload.match(/^relay:\/\/stream\/(.+)$/)
            if (match) {
              viewer.loadStream(match[1]).then(() => setTab('stream-viewer'))
            }
          }
        })
        const urls = await getCurrent()
        if (urls) {
          for (const payload of urls) {
            const match = payload.match(/^relay:\/\/stream\/(.+)$/)
            if (match) {
              await viewer.loadStream(match[1])
              setTab('stream-viewer')
            }
          }
        }
      } catch {
        // deep-link plugin may not be available
      }
    }

    initDeepLink()

    // Listen for AI degradation toast from background thread
    let unlistenAi: (() => void) | undefined
    let unlistenCapture: (() => void) | undefined
    let unlistenSettings: (() => void) | undefined
    async function initListeners() {
      try {
        unlistenAi = await listen<string>('relay://ai-degraded', (evt) => {
          toast({ message: evt.payload, type: 'error' })
        })
      } catch { /* event plugin may not be available */ }
      try {
        unlistenCapture = await listen('relay://request-capture', () => {
          capture.handleCapture()
        })
      } catch { /* */ }
      try {
        unlistenSettings = await listen('relay://open-settings', () => {
          setTab('settings')
        })
      } catch { /* */ }
    }
    initListeners()

    return () => {
      if (unlistenAi) unlistenAi()
      if (unlistenCapture) unlistenCapture()
      if (unlistenSettings) unlistenSettings()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  function handleSelectStream(s: StreamInfo) {
    streams.selectStream(s).then(() => setTab('stream-detail'))
  }

  return (
    <div className="app">
      <h1>Gearbox Relay</h1>
      <p className="subtitle">Capture, enrich, publish, and subscribe.</p>

      <TabBar
        active={tab}
        onTabChange={(t: Tab) => {
          if (t === 'history') history.refresh()
          setTab(t)
        }}
        onLoadStreams={streams.loadStreams}
        onLoadFeed={feed.loadFeed}
        onLoadProfile={settings.loadProfile}
      />

      {tab === 'capture' && (
        <>
          <CapturePanel
            streamingSummary={capture.streamingSummary}
            streamingTags={capture.streamingTags}
            streamingConnection={capture.streamingConnection}
            isStreaming={capture.isStreaming}
            lastResult={capture.lastResult}
            error={capture.error}
            onCapture={capture.handleCapture}
            modelStatus={model.status}
          />
          {capture.lastResult && streams.selectedStream && (
            <div style={{ marginTop: '0.5rem' }}>
              <button
                onClick={() => streams.addHighlight(capture.lastResult!.id)}
              >
                Add to Stream: {streams.selectedStream.title}
              </button>
            </div>
          )}
        </>
      )}

      {tab === 'search' && (
        <SearchPanel
          query={search.query}
          onQueryChange={search.setQuery}
          results={search.results}
          isSearching={search.isSearching}
          error={search.error}
          selectedIndex={search.selectedIndex}
          selectedStream={streams.selectedStream}
          onSearch={search.handleSearch}
          onKeyDown={search.handleKeyDown}
          onAddToStream={streams.addHighlight}
          semantic={search.semantic}
          onSemanticChange={search.setSemantic}
          filters={search.filters}
          onFiltersChange={search.setFilters}
        />
      )}

      {tab === 'history' && <HistoryPanel />}

      {tab === 'review' && <ReviewPanel />}

      {tab === 'following' && <FollowingFeed />}

      {tab === 'feed' && <FeedPanel feed={feed.feed} error={feed.error} />}

      {tab === 'streams' && (
        <StreamsPanel
          streams={streams.streams}
          createTitle={streams.createTitle}
          onCreateTitleChange={streams.setCreateTitle}
          createDesc={streams.createDesc}
          onCreateDescChange={streams.setCreateDesc}
          error={streams.error}
          onCreate={streams.createStream}
          onSelect={handleSelectStream}
          onDelete={streams.deleteStream}
        />
      )}

      {tab === 'stream-detail' && streams.selectedStream && (
        <StreamDetailPanel
          stream={streams.selectedStream}
          highlights={streams.streamHighlights}
          shareLink={streams.shareLink}
          copied={streams.copied}
          error={streams.error}
          onBack={() => setTab('streams')}
          onShare={streams.share}
          onCopyLink={streams.copyLink}
          onRemove={streams.removeHighlight}
        />
      )}

      {tab === 'stream-viewer' && (
        <StreamViewerPanel
          stream={viewer.stream}
          highlights={viewer.highlights}
          error={viewer.error}
          curatorProfile={viewer.curatorProfile}
          isSubscribed={viewer.isSubscribed}
          onToggleSubscription={viewer.toggleSubscription}
        />
      )}

      {tab === 'settings' && (
        <SettingsPanel
          profile={settings.profile}
          emailInput={settings.emailInput}
          onEmailChange={settings.setEmailInput}
          message={settings.message}
          onSaveEmail={settings.saveEmail}
          autoCapture={settings.autoCapture}
          onAutoCaptureChange={settings.setAutoCapture}
          telemetryEnabled={settings.telemetryEnabled}
          onTelemetryChange={settings.setTelemetryEnabled}
          theme={theme}
          onThemeChange={handleThemeChange}
        />
      )}

      <OnboardingModal />
      <ToastContainer />
    </div>
  )
}
