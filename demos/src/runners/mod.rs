//! Demo runners for executing quantum algorithms.

pub mod benchmark;
pub mod mitigation;
pub mod orchestrator;
pub mod qaoa;
pub mod scheduled;
pub mod vqe;

pub use benchmark::{
    benchmark_qaoa, benchmark_vqe, qaoa_scaling_benchmark, vqe_scaling_benchmark,
    BackendComparison, BenchmarkConfig, BenchmarkResult, BenchmarkTimer,
};
pub use mitigation::{
    zero_noise_extrapolation, MeasurementMitigator, MitigationConfig, ZneResult,
};
pub use orchestrator::run_multi_demo;
pub use qaoa::{QaoaResult, QaoaRunner};
pub use scheduled::{ScheduledDemoConfig, ScheduledDemoResult, ScheduledRunner};
pub use vqe::{VqeResult, VqeRunner};
