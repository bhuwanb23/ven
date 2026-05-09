//! Rank and format resolution suggestions (extends raw options from conflicts).

use crate::intelligence::graph::{ResolutionOption, SimulationResult};

pub fn print_conflict_report(result: &SimulationResult) {
    use colored::Colorize;
    if result.conflict_chains.is_empty() && result.engine_incompatibilities.is_empty() {
        return;
    }
    println!("\n  {}", "Dependency intelligence — issues".bold().red());
    for chain in &result.conflict_chains {
        println!("  {} {}", "✗".red(), chain.package.bold());
        for step in &chain.steps {
            println!("      └─ {}", step.dimmed());
        }
    }
    for inc in &result.engine_incompatibilities {
        println!(
            "  {} {}@{} requires Node {} (you: {})",
            "✗".red(),
            inc.package,
            inc.version,
            inc.required_node,
            inc.current_node
        );
    }
    if !result.suggestions.is_empty() {
        println!("\n  {}", "Resolution options".bold().cyan());
        for opt in &result.suggestions {
            println!("    [{}] {}", opt.id, opt.label);
        }
    }
}

pub fn merge_suggestions(result: &mut SimulationResult) {
    let mut id = 1u32;
    for opt in &mut result.suggestions {
        opt.id = id;
        id += 1;
    }
}

pub fn apply_choice(result: &SimulationResult, choice: u32) -> Option<&ResolutionOption> {
    result.suggestions.iter().find(|o| o.id == choice)
}
