# Contributing to Gearbox Relay

Relay is a local-first, AI-native personal knowledge pipeline. We welcome contributions that improve correctness, privacy, performance, or developer experience.

## Getting Started

```bash
# Prerequisites: Rust, Node.js (20+), pnpm, Flutter 3.48+
git clone https://github.com/AWRMemo/gearbox-client.git
cd gearbox-client

pnpm install
cargo build
pnpm tauri dev
```

Full setup guide: `docs/development-setup.md`

## Repository Structure

This is the **public snapshot** of the Gearbox Relay client. The development workflow is:

1. All active development happens on the private `gearbox` repository
2. At stable milestones, the client code is snapshot to this public repo
3. PRs submitted here will be reviewed and cherry-picked back to the private repo

Submit PRs against `main` on this repository. We review weekly.

## Code Conventions

**Rust:**
- Tauri commands return `Result<T, String>` — delegate business logic to service modules
- `#[tauri::command]` handlers live in `src-tauri/src/commands/` and must be thin wrappers
- All AI operations use `tauri::ipc::Channel<String>` for non-blocking IPC
- Error types use `thiserror`; convert to `String` at the command boundary

**React:**
- Named exports only
- Components under 200 lines
- State in local hooks; no global store for v1
- Tests use Vitest with `jsdom` environment

**Conventional Commits:** `feat:`, `fix:`, `chore:`, `docs:`, `test:`

See `AGENTS.md` for the full sprint history and locked architecture decisions.

## Testing

```bash
# Rust fast tests
cargo test --workspace --exclude relay-sync-server

# Rust slow tests (LanceDB, ONNX, E2E)
cargo test --workspace --exclude relay-sync-server -- --ignored

# Frontend tests
pnpm test

# Lint
cargo clippy --all-targets --workspace --exclude relay-sync-server -- -D warnings
pnpm lint
```

All PRs must pass: `cargo clippy`, `cargo test`, `pnpm test`, `pnpm lint`.

## Never Do

- Never use a cloud AI API (OpenAI, Anthropic, etc.)
- Never commit `.env` files or credentials
- Never modify `src-tauri/src/ai/fallback.rs` without updating its unit tests
- Never introduce Yjs or any CRDT library
- Never bind local servers to `0.0.0.0` — use `127.0.0.1:0` only

## License

Apache 2.0. All contributions are licensed under the same terms.
