//! Unified dependency intelligence: graph build, constraints, suggestions, persistence.

pub mod adapters;
pub mod conflicts;
pub mod display;
pub mod engine;
pub mod graph;
pub mod store;
pub mod suggestions;

pub use engine::DependencyIntelligenceService;
pub use graph::*;
