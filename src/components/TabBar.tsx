import type { Tab } from '../types'

interface TabBarProps {
  active: Tab
  onTabChange: (tab: Tab) => void
  onLoadStreams: () => void
  onLoadFeed: () => void
  onLoadProfile: () => void
  onLoadFollowing?: () => void
}

const TABS: { key: Tab; label: string }[] = [
  { key: 'capture', label: 'Capture' },
  { key: 'history', label: 'History' },
  { key: 'review', label: 'Review' },
  { key: 'following', label: 'Following' },
  { key: 'search', label: 'Search' },
  { key: 'feed', label: 'Feed' },
  { key: 'streams', label: 'Streams' },
  { key: 'settings', label: 'Settings' },
]

export function TabBar({ active, onTabChange, onLoadStreams, onLoadFeed, onLoadProfile, onLoadFollowing }: TabBarProps) {
  return (
    <div className="tab-bar">
      {TABS.map(({ key, label }) => (
        <button
          key={key}
          className={active === key ? 'tab-active' : ''}
          onClick={() => {
            if (key === 'streams') onLoadStreams()
            if (key === 'feed') onLoadFeed()
            if (key === 'settings') onLoadProfile()
            if (key === 'following') onLoadFollowing?.()
            onTabChange(key)
          }}
        >
          {label}
        </button>
      ))}
    </div>
  )
}
