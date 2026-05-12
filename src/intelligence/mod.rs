//! Unified dependency intelligence: graph build, constraints, suggestions, persistence.

pub mod adapters;
pub mod conflicts;
pub mod display;
pub mod drift;
pub mod engine;
pub mod graph;
pub mod store;
pub mod suggestions;
pub mod ven_lock;

pub use engine::DependencyIntelligenceService;
pub use graph::*;
