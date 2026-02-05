//! Quantum circuit generators for demos.

pub mod grover;
pub mod qaoa;
pub mod vqe;

pub use grover::grover_circuit;
pub use qaoa::{
    graph_aware_initial_parameters, initial_parameters_with_strategy, qaoa_circuit, InitStrategy,
    ParameterBounds,
};
pub use vqe::two_local_ansatz;
