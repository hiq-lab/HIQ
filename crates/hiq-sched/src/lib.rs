//! HIQ HPC Scheduler with SLURM integration.
//!
//! This crate provides a scheduler for submitting quantum circuit jobs to HPC clusters
//! via SLURM, with support for priority-based queuing, resource matching, batch submission,
//! and job dependencies through workflow DAGs.
//!
//! # Features
//!
//! - **SLURM Integration**: Submit jobs via `sbatch`, track via `squeue`/`sacct`
//! - **Priority Queuing**: Jobs with higher priority execute first
//! - **Resource Matching**: Match circuits to backends based on qubit count, topology
//! - **Batch Submission**: Submit multiple circuits as a single batch job
//! - **Job Dependencies**: Define workflows where jobs depend on other jobs completing
//!
//! # Example
//!
//! ```ignore
//! use hiq_sched::{HpcScheduler, ScheduledJob, Priority, ResourceRequirements};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let scheduler = HpcScheduler::new(config).await?;
//!
//!     // Submit a single job
//!     let job = ScheduledJob::new("my_job", circuit)
//!         .with_priority(Priority::high())
//!         .with_shots(1000);
//!     let job_id = scheduler.submit(job).await?;
//!
//!     // Wait for result
//!     let result = scheduler.wait(&job_id).await?;
//!     println!("Result: {:?}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod job;
pub mod queue;
pub mod scheduler;
pub mod workflow;
pub mod matcher;
pub mod slurm;
pub mod persistence;

// Re-exports
pub use error::{SchedError, SchedResult};
pub use job::{
    CircuitSpec, Priority, ResourceRequirements, ScheduledJob, ScheduledJobId, ScheduledJobStatus,
    TopologyPreference,
};
pub use queue::PriorityQueue;
pub use scheduler::{HpcScheduler, Scheduler, SchedulerConfig};
pub use workflow::{Workflow, WorkflowBuilder, WorkflowId, WorkflowStatus};
pub use matcher::{MatchResult, ResourceMatcher};
pub use slurm::{SlurmAdapter, SlurmConfig};
pub use persistence::{JsonStore, SqliteStore, StateStore};
