//! Classical optimizers for variational algorithms.

pub mod cobyla;

pub use cobyla::{Cobyla, OptimizationResult};

/// Trait for classical optimizers.
pub trait Optimizer {
    /// Minimize the objective function.
    ///
    /// # Arguments
    /// * `objective` - The function to minimize, takes parameters and returns value
    /// * `initial_params` - Starting point
    ///
    /// # Returns
    /// Optimization result with optimal parameters and value.
    fn minimize<F>(&self, objective: F, initial_params: Vec<f64>) -> OptimizationResult
    where
        F: FnMut(&[f64]) -> f64;
}
