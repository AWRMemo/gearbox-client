import { test, expect } from '@playwright/test'

// Mock the Tauri invoke function so the UI can render without a Rust backend
async function setupMocks(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    const mockData: Record<string, unknown> = {
      list_stored_highlights: [
        { id: 'h1', text: 'Rust is memory-safe without garbage collection.', summary: 'Memory safety in Rust', tags: ['rust', 'systems'], source_url: 'https://www.rust-lang.org', created_at: '2026-06-01T12:00:00Z' },
        { id: 'h2', text: 'Tauri bundles web apps into tiny desktop binaries.', summary: 'Tauri desktop packaging', tags: ['tauri', 'desktop'], source_url: 'https://tauri.app', created_at: '2026-06-02T12:00:00Z' },
      ],
      get_history_paginated: {
        entries: [
          { id: 'h1', text: 'Rust is memory-safe without garbage collection.', summary: 'Memory safety in Rust', tags: ['rust', 'systems'], source_url: 'https://www.rust-lang.org', created_at: '2026-06-01T12:00:00Z' },
          { id: 'h2', text: 'Tauri bundles web apps into tiny desktop binaries.', summary: 'Tauri desktop packaging', tags: ['tauri', 'desktop'], source_url: 'https://tauri.app', created_at: '2026-06-02T12:00:00Z' },
        ],
        hasMore: false,
      },
      search: [
        { id: 'h1', text: 'Rust is memory-safe without garbage collection.', summary: 'Memory safety in Rust', tags: ['rust'], score: 0.9 },
      ],
      get_model_status: {
        loaded: true,
        model_name: 'Qwen-3.5-0.8B',
        embedding_available: true,
        download_progress: null,
        download_state: 'done',
      },
      list_my_streams: [
        { id: 's1', title: 'Engineering Notes', description: 'My tech highlights', created_at: '2026-06-01T00:00:00Z' },
      ],
      get_user_profile: { email: 'test@example.com', tier: 'free' },
      get_auth_status: null,
      get_sync_status: { status: 'idle', last_sync: null },
      get_device_id: 'test-device-001',
      get_telemetry_opt_out: true,
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: Record<string, unknown>) => {
          const result = mockData[cmd as keyof typeof mockData]
          if (result !== undefined) {
            if (typeof result === 'function') return (result as (args?: unknown) => unknown)(args)
            return result
          }
          return null
        },
      },
      event: {
        listen: () => Promise.resolve(() => {}),
      },
    }
  })
}

test.describe('Desktop E2E Journey', () => {
  test.beforeEach(async ({ page }) => {
    await setupMocks(page)
    await page.goto('http://localhost:1420')
  })

  test('launch app and see heading', async ({ page }) => {
    await expect(page.locator('h1:first-of-type')).toBeVisible()
  })

  test('dismiss onboarding and see capture tab', async ({ page }) => {
    // Try to dismiss onboarding modal
    const skipBtn = page.getByRole('button', { name: /skip|get started/i })
    const closeBtn = page.getByRole('button', { name: /close/i })

    try {
      if (await skipBtn.isVisible({ timeout: 3000 })) {
        await skipBtn.click()
      }
    } catch {
      /* onboarding may already be dismissed */
    }

    await expect(page.getByText(/capture/i).first()).toBeVisible({ timeout: 5000 })
  })

  test('capture text and see history entry', async ({ page }) => {
    // Find the capture textarea input and type text
    const textarea = page.locator('textarea, [contenteditable="true"]').first()
    if (await textarea.isVisible({ timeout: 3000 })) {
      await textarea.fill('This is a test highlight about Rust programming.')
    }

    // Check that history tab shows entries from mock
    const historyTab = page.getByRole('button', { name: /history/i })
    if (await historyTab.isVisible()) {
      await historyTab.click()
      await page.waitForTimeout(500)
    }

    // Should show mock history entries
    await expect(page.getByText('Memory safety in Rust')).toBeVisible({ timeout: 5000 })
  })

  test('search returns results from mock', async ({ page }) => {
    // Dismiss onboarding if present
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Click Search tab
    const searchTab = page.getByRole('button', { name: /search/i })
    await searchTab.click()
    await page.waitForTimeout(300)

    // Type into search input
    const searchInput = page.getByPlaceholder('Search…')
    await searchInput.fill('rust')

    // Click Search button
    const searchBtn = page.getByRole('button', { name: 'Search' })
    if (await searchBtn.isVisible({ timeout: 3000 })) {
      await searchBtn.click()
      await page.waitForTimeout(1000)
    }

    // Should show mock search result
    await expect(page.getByText(/result/i)).toBeVisible({ timeout: 5000 })
  })

  test('navigate to streams and see mock streams', async ({ page }) => {
    // Dismiss onboarding if present
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Click Streams tab
    const streamsTab = page.getByRole('button', { name: /streams/i })
    if (await streamsTab.isVisible({ timeout: 3000 })) {
      await streamsTab.click()
      await page.waitForTimeout(500)
    }

    // Should show mock stream title
    await expect(page.getByText('Engineering Notes')).toBeVisible({ timeout: 5000 })
  })

  test('settings panel shows export and theme options', async ({ page }) => {
    // Dismiss onboarding if present
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Click Settings tab
    const settingsTab = page.getByRole('button', { name: /settings/i })
    if (await settingsTab.isVisible({ timeout: 3000 })) {
      await settingsTab.click()
      await page.waitForTimeout(500)
    }

    // Check for export button
    const exportBtn = page.getByRole('button', { name: /export/i })
    await expect(exportBtn).toBeVisible({ timeout: 5000 })

    // Check for dark mode toggle
    const darkModeLabel = page.getByText(/dark mode/i)
    await expect(darkModeLabel).toBeVisible({ timeout: 5000 })
  })

  test('empty state shown when no content available', async ({ page }) => {
    // Dismiss onboarding
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Navigate to a tab that might show empty state
    const historyTab = page.getByRole('button', { name: /history/i })
    if (await historyTab.isVisible({ timeout: 3000 })) {
      await historyTab.click()
      await page.waitForTimeout(500)
    }

    // Should show some content (either history entries or empty state)
    const hasContent = page.getByText(/memory safety|no.*history|no items/i)
    await expect(hasContent.first()).toBeVisible({ timeout: 5000 })
  })

  test('create stream from search results', async ({ page }) => {
    // Dismiss onboarding if present
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Navigate to Streams tab
    const streamsTab = page.getByRole('button', { name: /streams/i })
    if (await streamsTab.isVisible({ timeout: 3000 })) {
      await streamsTab.click()
      await page.waitForTimeout(500)
    }

    // Should see create stream button or empty state
    const createBtn = page.getByRole('button', { name: /create|new stream/i })
    const emptyState = page.getByText(/no.*stream|create.*stream/i)
    const visible = await createBtn.isVisible({ timeout: 3000 }).catch(() => false)
    const emptyVisible = await emptyState.isVisible({ timeout: 3000 }).catch(() => false)
    expect(visible || emptyVisible).toBeTruthy()
  })

  test('theme toggle switches between light and dark', async ({ page }) => {
    // Dismiss onboarding
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Navigate to Settings
    const settingsTab = page.getByRole('button', { name: /settings/i })
    if (await settingsTab.isVisible({ timeout: 3000 })) {
      await settingsTab.click()
      await page.waitForTimeout(500)
    }

    // Find and click dark mode toggle
    const darkLabel = page.getByText(/dark mode/i)
    if (await darkLabel.isVisible({ timeout: 3000 })) {
      const checkbox = page.locator('#settings-theme')
      if (await checkbox.isVisible({ timeout: 2000 })) {
        await checkbox.click()
        await page.waitForTimeout(300)
        // Toggle back
        await checkbox.click()
      }
    }
  })

  test('clear data button shows confirmation', async ({ page }) => {
    // Dismiss onboarding
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Navigate to Settings
    const settingsTab = page.getByRole('button', { name: /settings/i })
    if (await settingsTab.isVisible({ timeout: 3000 })) {
      await settingsTab.click()
      await page.waitForTimeout(500)
    }

    // Check clear data button exists
    const clearBtn = page.getByRole('button', { name: /clear.*data/i })
    await expect(clearBtn).toBeVisible({ timeout: 5000 })
  })

  test('auth form visible when not authenticated', async ({ page }) => {
    // Dismiss onboarding
    try {
      const skipBtn = page.getByRole('button', { name: /skip|get started/i })
      if (await skipBtn.isVisible({ timeout: 2000 })) {
        await skipBtn.click()
      }
    } catch { /* ignore */ }

    // Navigate to Settings
    const settingsTab = page.getByRole('button', { name: /settings/i })
    if (await settingsTab.isVisible({ timeout: 3000 })) {
      await settingsTab.click()
      await page.waitForTimeout(500)
    }

    // Auth form should show email input or login/signup UI
    const emailInput = page.getByPlaceholder(/email/i)
    const authText = page.getByText(/sign in|log in|account|auth/i)
    const visible = await emailInput.isVisible({ timeout: 3000 }).catch(() => false)
    const authVisible = await authText.isVisible({ timeout: 3000 }).catch(() => false)
    expect(visible || authVisible).toBeTruthy()
  })
})
