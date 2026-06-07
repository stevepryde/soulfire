//! Worlds: the turn engine, adventure-state schema, memory ladder, state-patch
//! validator, diff/full reconciliation, `/gm` flow, and the world builder
//! (`WORLD`).
//!
//! Built in testable units: `state_patch` (patch application + validator),
//! `memory` (the three-store ladder), `response` (tolerant update parsing),
//! `prompts` (verbatim game-master prompt text), and `engine` (the turn engine).

pub mod engine;
pub mod input;
pub mod memory;
pub mod prompts;
pub mod response;
pub mod state_patch;

pub use engine::{TurnOutcome, WorldEngine};
pub use input::{TurnInput, parse_turn_input};
pub use memory::{SignificantEvent, SignificantEventUpdates};
pub use state_patch::{PatchOp, PatchResult, StatePatch, apply_patches};
