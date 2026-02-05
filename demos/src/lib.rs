//! HIQ Demo Suite
//!
//! This crate provides demonstrations of HiQ's HPC-quantum orchestration
//! capabilities using various quantum algorithms:
//!
//! - **Grover's Search**: Simple baseline demo
//! - **VQE (Variational Quantum Eigensolver)**: Hybrid classical-quantum workflow
//! - **QAOA (Quantum Approximate Optimization)**: Graph optimization
//! - **Multi-Job Orchestration**: Parallel job management showcase

pub mod circuits;
pub mod optimizers;
pub mod problems;
pub mod runners;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Create a progress bar for demo operations.
pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Print a demo header.
pub fn print_header(title: &str) {
    println!();
    println!("{}", style("═".repeat(60)).cyan());
    println!("{}", style(format!("  {}", title)).cyan().bold());
    println!("{}", style("═".repeat(60)).cyan());
    println!();
}

/// Print a demo section.
pub fn print_section(title: &str) {
    println!();
    println!("{}", style(format!("▶ {}", title)).green().bold());
    println!("{}", style("─".repeat(40)).dim());
}

/// Print a result line.
pub fn print_result(label: &str, value: impl std::fmt::Display) {
    println!("  {} {}", style(format!("{}:", label)).dim(), value);
}

/// Print a success message.
pub fn print_success(message: &str) {
    println!("{} {}", style("✓").green().bold(), message);
}

/// Print an info message.
pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ").blue(), message);
}
