//! Demo runners for executing quantum algorithms.

pub mod orchestrator;
pub mod qaoa;
pub mod vqe;

pub use orchestrator::run_multi_demo;
pub use qaoa::{QaoaResult, QaoaRunner};
pub use vqe::{VqeResult, VqeRunner};
