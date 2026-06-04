import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['dist', 'target', 'src-tauri/target', 'relay-core/target', 'relay_sync_server/target', '**/*.test.{ts,tsx}', '**/*.spec.{ts,tsx}', 'extensions/']),
  js.configs.recommended,
  ...(Array.isArray(tseslint.configs.recommended) ? tseslint.configs.recommended : [tseslint.configs.recommended]),
  reactHooks.configs.flat.recommended,
  reactRefresh.configs.vite,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      globals: globals.browser,
    },
  },
])
