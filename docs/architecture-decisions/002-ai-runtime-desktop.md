# ADR 002: Desktop AI Runtime Selection

**Status:** Accepted
**Date:** 2026-05-19
**Supersedes:** PRD §6.2 (cross-platform claim)

## Context

The PRD originally specified Cactus SDK as the cross-platform AI runtime for both desktop and mobile. During implementation, we discovered Cactus SDK has no production support for Rust/Tauri/Windows. We needed an immediate, working alternative for the desktop MVP.

## Decision

Use `llama-cpp-2` (Rust bindings for llama.cpp) as the on-device AI runtime on desktop. The same GGUF model files are loaded in-process with no server dependency.

## Consequences

- Desktop MVP can run real AI with zero cloud costs and full privacy.
- Mobile will require a different runtime (Cactus SDK or MLX Swift), but the model files and prompts remain identical.
- The PRD Technology Stack table has been updated to reflect this separation.
- The portable Rust core can be reused on mobile via FFI/JNI.
