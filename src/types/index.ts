export interface CaptureResult {
  id: string
  summary: string
  tags: string[]
  connection_suggestion: string | null
  source_url: string | null
  source_title: string | null
  source_author: string | null
}

export interface HistoryEntry {
  id: string
  text: string
  result: CaptureResult
  timestamp: number
}

export interface SearchResult {
  id: string
  summary: string
  tags: string[]
  text: string
  score: number
}

export interface StreamInfo {
  id: string
  user_id: string
  title: string
  description: string
  is_public: boolean
  created_at: string
  updated_at: string
}

export interface StreamHighlight {
  id: string
  text: string
  summary: string
  tags: string[]
  source_url: string | null
}

export interface FeedHighlight {
  id: string
  text: string
  summary: string
  tags: string[]
  source_url: string | null
  stream_id: string
  stream_title: string
}

export interface ListedHighlight {
  id: string
  text: string
  source_url: string | null
  source_title: string | null
  source_author: string | null
  summary: string
  tags: string[]
  created_at: string
  connection_suggestion: string | null
}

export interface UserProfile {
  user_id: string
  email: string | null
  display_name: string | null
  tier: string
  created_at: string
}

export interface ModelStatus {
  loaded: boolean
  model_name: string | null
  embedding_available: boolean
  download_progress: number | null
  download_state: 'idle' | 'downloading' | 'done' | 'error'
}

export type Tab =
  | 'capture'
  | 'streams'
  | 'stream-detail'
  | 'stream-viewer'
  | 'feed'
  | 'settings'
  | 'history'
  | 'following'
  | 'search'
  | 'review'

export interface AuthStatus {
  signed_in: boolean
  email: string | null
  server_url: string | null
}

export interface SyncStatus {
  last_sync: string | null
  status: 'idle' | 'syncing' | 'offline' | 'error'
  pending_conflicts: number
}

export interface Conflict {
  id: string
  record_type: string
  record_id: string
  local_version: string | null
  remote_version: string | null
  created_at: string
}

export type ConflictResolution = 'accept_remote' | 'keep_local'

export interface SearchFilter {
  dateFrom?: string
  dateTo?: string
  tags?: string[]
  sourceDomain?: string
}
