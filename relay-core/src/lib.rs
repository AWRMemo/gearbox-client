pub mod ai;
pub mod config;
pub mod db;
pub mod error;
pub mod review;
pub mod sync;
pub mod telemetry;
pub mod types;

// NOTE: FFI surface for flutter_rust_bridge is intentionally kept OUT of relay-core.
// The mobile bridge crate (relay-mobile-bridge) will depend on relay-core and
// add FRB proc-macro annotations there. This keeps relay-core pure Rust and
// mobile-agnostic.
