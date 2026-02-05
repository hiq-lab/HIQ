//! Classical Optimizers for VQE
//!
//! This module provides classical optimization algorithms for
//! the variational loop in VQE.

use std::f64::consts::PI;

/// Trait for classical optimizers
pub trait Optimizer {
    /// Take one optimization step
    fn step(&mut self, params: &[f64], cost: f64) -> Vec<f64>;

    /// Reset the optimizer state
    fn reset(&mut self);
}

/// COBYLA-inspired optimizer (Constrained Optimization BY Linear Approximations)
///
/// This is a derivative-free optimizer suitable for noisy quantum
/// cost function evaluations.
pub struct CobylaOptimizer {
    /// Number of parameters
    num_params: usize,

    /// Parameter bounds
    bounds: Vec<(f64, f64)>,

    /// Convergence tolerance
    tolerance: f64,

    /// Current step size (trust region radius)
    rho: f64,

    /// Minimum step size
    rho_min: f64,

    /// Step counter
    iteration: usize,

    /// Best parameters found
    best_params: Option<Vec<f64>>,

    /// Best cost found
    best_cost: f64,

    /// Simplex for derivative-free search
    simplex: Vec<Vec<f64>>,

    /// Simplex costs
    simplex_costs: Vec<f64>,
}

impl CobylaOptimizer {
    /// Create a new COBYLA optimizer
    pub fn new(num_params: usize) -> Self {
        Self {
            num_params,
            bounds: vec![(-PI, PI); num_params],
            tolerance: 1e-4,
            rho: 0.5,
            rho_min: 1e-4,
            iteration: 0,
            best_params: None,
            best_cost: f64::MAX,
            simplex: Vec::new(),
            simplex_costs: Vec::new(),
        }
    }

    /// Set parameter bounds
    pub fn with_bounds(mut self, bounds: Vec<(f64, f64)>) -> Self {
        assert_eq!(bounds.len(), self.num_params);
        self.bounds = bounds;
        self
    }

    /// Set convergence tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// Project parameters onto bounds
    fn project(&self, params: &[f64]) -> Vec<f64> {
        params
            .iter()
            .enumerate()
            .map(|(i, &p)| p.clamp(self.bounds[i].0, self.bounds[i].1))
            .collect()
    }

    /// Initialize simplex around current point
    fn init_simplex(&mut self, center: &[f64]) {
        self.simplex.clear();
        self.simplex_costs.clear();

        // Add center point
        self.simplex.push(center.to_vec());
        self.simplex_costs.push(f64::MAX);

        // Add vertices along each axis
        for i in 0..self.num_params {
            let mut vertex = center.to_vec();
            vertex[i] += self.rho;
            self.simplex.push(self.project(&vertex));
            self.simplex_costs.push(f64::MAX);
        }
    }
}

impl Optimizer for CobylaOptimizer {
    fn step(&mut self, params: &[f64], cost: f64) -> Vec<f64> {
        self.iteration += 1;

        // Update best if improved
        if cost < self.best_cost {
            self.best_cost = cost;
            self.best_params = Some(params.to_vec());
        }

        // Initialize simplex on first iteration
        if self.simplex.is_empty() {
            self.init_simplex(params);
            self.simplex_costs[0] = cost;
            // Return second simplex point for evaluation
            return self.simplex[1].clone();
        }

        // Find which simplex point this cost corresponds to
        let eval_idx = (self.iteration - 1) % (self.num_params + 1);
        if eval_idx < self.simplex_costs.len() {
            self.simplex_costs[eval_idx] = cost;
        }

        // After evaluating all simplex points, perform update
        if self.iteration % (self.num_params + 1) == 0 {
            // Sort simplex by cost
            let mut indices: Vec<usize> = (0..self.simplex.len()).collect();
            indices.sort_by(|&a, &b| {
                self.simplex_costs[a]
                    .partial_cmp(&self.simplex_costs[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Compute centroid of best points (excluding worst)
            let best_indices = &indices[..self.num_params];
            let mut centroid = vec![0.0; self.num_params];
            for &idx in best_indices {
                for (j, &p) in self.simplex[idx].iter().enumerate() {
                    centroid[j] += p;
                }
            }
            for c in &mut centroid {
                *c /= self.num_params as f64;
            }

            // Reflect worst point through centroid
            let worst_idx = *indices.last().unwrap();
            let worst = &self.simplex[worst_idx];
            let reflected: Vec<f64> = centroid
                .iter()
                .zip(worst.iter())
                .map(|(&c, &w)| 2.0 * c - w)
                .collect();

            // Update simplex
            self.simplex[worst_idx] = self.project(&reflected);

            // Reduce step size if converging
            let spread: f64 = self
                .simplex
                .iter()
                .flat_map(|v| v.iter())
                .map(|&x| x.abs())
                .fold(0.0, f64::max);

            if spread < self.tolerance && self.rho > self.rho_min {
                self.rho *= 0.5;
                // Re-initialize simplex with smaller radius
                if let Some(best) = self.best_params.clone() {
                    self.init_simplex(&best);
                }
            }

            // Return best point for next evaluation
            return self.simplex[indices[0]].clone();
        }

        // Return next simplex point for evaluation
        let next_idx = self.iteration % (self.num_params + 1);
        if next_idx < self.simplex.len() {
            self.simplex[next_idx].clone()
        } else {
            self.best_params.clone().unwrap_or_else(|| params.to_vec())
        }
    }

    fn reset(&mut self) {
        self.iteration = 0;
        self.best_params = None;
        self.best_cost = f64::MAX;
        self.simplex.clear();
        self.simplex_costs.clear();
        self.rho = 0.5;
    }
}

/// Simple gradient descent optimizer (for comparison)
pub struct GradientDescentOptimizer {
    /// Learning rate
    learning_rate: f64,

    /// Number of parameters
    num_params: usize,

    /// Previous parameters for finite difference gradient
    prev_params: Option<Vec<f64>>,

    /// Previous cost for gradient estimation
    prev_cost: f64,

    /// Gradient estimate
    gradient: Vec<f64>,

    /// Current parameter index for gradient estimation
    gradient_idx: usize,

    /// Iteration counter
    iteration: usize,
}

impl GradientDescentOptimizer {
    /// Create a new gradient descent optimizer
    pub fn new(num_params: usize) -> Self {
        Self {
            learning_rate: 0.1,
            num_params,
            prev_params: None,
            prev_cost: f64::MAX,
            gradient: vec![0.0; num_params],
            gradient_idx: 0,
            iteration: 0,
        }
    }

    /// Set learning rate
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }
}

impl Optimizer for GradientDescentOptimizer {
    fn step(&mut self, params: &[f64], cost: f64) -> Vec<f64> {
        self.iteration += 1;

        // Simple parameter-shift rule gradient estimation
        // Every 2*num_params iterations, we complete a gradient estimate

        let cycle_pos = (self.iteration - 1) % (2 * self.num_params + 1);

        if cycle_pos == 0 {
            // Start new gradient estimation cycle
            self.prev_params = Some(params.to_vec());
            self.prev_cost = cost;
            self.gradient_idx = 0;

            // Perturb first parameter positively
            let mut new_params = params.to_vec();
            new_params[0] += PI / 2.0;
            return new_params;
        }

        let param_idx = (cycle_pos - 1) / 2;
        let is_positive = (cycle_pos - 1) % 2 == 0;

        if is_positive {
            // Received cost for positive shift
            self.gradient[param_idx] = cost;

            // Now do negative shift
            if let Some(ref base) = self.prev_params {
                let mut new_params = base.clone();
                new_params[param_idx] -= PI / 2.0;
                return new_params;
            }
        } else {
            // Received cost for negative shift
            // Complete gradient for this parameter
            self.gradient[param_idx] = (self.gradient[param_idx] - cost) / 2.0;

            if param_idx + 1 < self.num_params {
                // Move to next parameter
                if let Some(ref base) = self.prev_params {
                    let mut new_params = base.clone();
                    new_params[param_idx + 1] += PI / 2.0;
                    return new_params;
                }
            } else {
                // All gradients computed, do update
                if let Some(ref base) = self.prev_params {
                    let new_params: Vec<f64> = base
                        .iter()
                        .zip(self.gradient.iter())
                        .map(|(&p, &g)| p - self.learning_rate * g)
                        .collect();
                    return new_params;
                }
            }
        }

        params.to_vec()
    }

    fn reset(&mut self) {
        self.iteration = 0;
        self.prev_params = None;
        self.prev_cost = f64::MAX;
        self.gradient = vec![0.0; self.num_params];
        self.gradient_idx = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cobyla_creation() {
        let opt = CobylaOptimizer::new(2);
        assert_eq!(opt.num_params, 2);
    }

    #[test]
    fn test_cobyla_step() {
        let mut opt = CobylaOptimizer::new(1);
        let params = vec![0.0];
        let new_params = opt.step(&params, 1.0);

        // Should return different parameters
        assert!(!new_params.is_empty());
    }

    #[test]
    fn test_gradient_descent_creation() {
        let opt = GradientDescentOptimizer::new(2).with_learning_rate(0.05);
        assert_eq!(opt.num_params, 2);
        assert!((opt.learning_rate - 0.05).abs() < 1e-10);
    }
}
