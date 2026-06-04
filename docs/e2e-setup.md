# E2E Desktop Tests

## Prerequisites
- Node.js 20+
- Chromium browser (Playwright will auto-install)

## Setup
```bash
# Install Playwright browsers
npx playwright install chromium

# Install project dependencies
pnpm install
```

## Running
```bash
# Start Tauri dev server in one terminal
pnpm dev

# In another terminal, run E2E tests
pnpm test:e2e
```

## CI
The E2E suite runs via `.github/workflows/e2e-desktop.yml` on push to `main` and Sprint branches. It requires a headful display environment (Xvfb on Linux).
