interface SourceMetaProps {
  source_url?: string | null
  source_title?: string | null
  source_author?: string | null
}

export function SourceMeta({ source_url, source_title }: SourceMetaProps) {
  const parts: string[] = []
  if (source_title) parts.push(source_title)
  if (source_url) parts.push(source_url)
  if (parts.length === 0) return null
  return (
    <p style={{ fontSize: '0.75rem', color: 'var(--text-subtle)', marginTop: '0.2rem' }}>
      {source_title && <span>{source_title} — </span>}
      {source_url && <span style={{ wordBreak: 'break-all' }}>{source_url}</span>}
    </p>
  )
}
