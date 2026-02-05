//! QAOA (Quantum Approximate Optimization Algorithm) circuits.
//!
//! QAOA is a variational algorithm for combinatorial optimization problems.
//! It alternates between cost and mixer unitaries with tunable parameters.

use hiq_ir::qubit::QubitId;
use hiq_ir::Circuit;
use std::f64::consts::PI;

use crate::problems::Graph;

/// Generate a QAOA circuit for the Max-Cut problem.
///
/// QAOA consists of:
/// 1. Initial state: |+⟩^n (uniform superposition)
/// 2. For each layer p:
///    - Cost unitary: exp(-i γ C) where C encodes the graph
///    - Mixer unitary: exp(-i β B) where B = Σ Xⱼ
///
/// # Arguments
/// * `graph` - The Max-Cut graph
/// * `gamma` - Cost parameters (one per layer)
/// * `beta` - Mixer parameters (one per layer)
///
/// # Returns
/// A QAOA circuit with measurements.
pub fn qaoa_circuit(graph: &Graph, gamma: &[f64], beta: &[f64]) -> Circuit {
    assert_eq!(gamma.len(), beta.len(), "gamma and beta must have same length");
    let p = gamma.len(); // Number of QAOA layers

    let n = graph.n_nodes;
    let mut circuit = Circuit::with_size("qaoa", n as u32, n as u32);

    // Step 1: Initialize |+⟩^n
    for q in 0..n {
        circuit.h(QubitId(q as u32)).unwrap();
    }

    // Step 2: Apply p layers of cost and mixer unitaries
    for layer in 0..p {
        // Cost unitary: exp(-i γ C)
        // For Max-Cut: C = -1/2 Σ_{(i,j)∈E} (1 - Z_i Z_j)
        // exp(-i γ C) = Π_{(i,j)∈E} exp(i γ/2 Z_i Z_j)
        apply_cost_unitary(&mut circuit, graph, gamma[layer]);

        // Mixer unitary: exp(-i β B) where B = Σ X_j
        // exp(-i β B) = Π_j exp(-i β X_j) = Π_j RX(2β)
        apply_mixer_unitary(&mut circuit, n, beta[layer]);
    }

    // Step 3: Measure all qubits
    circuit.measure_all().unwrap();

    circuit
}

/// Apply the cost unitary for Max-Cut.
///
/// For each edge (i,j), apply: RZZ(γ) = exp(-i γ/2 Z_i Z_j)
/// RZZ can be decomposed as: CNOT(i,j) · RZ(γ)[j] · CNOT(i,j)
fn apply_cost_unitary(circuit: &mut Circuit, graph: &Graph, gamma: f64) {
    for (i, j, weight) in &graph.edges {
        // RZZ(gamma * weight) decomposition
        let angle = gamma * weight;
        circuit.cx(QubitId(*i as u32), QubitId(*j as u32)).unwrap();
        circuit.rz(angle, QubitId(*j as u32)).unwrap();
        circuit.cx(QubitId(*i as u32), QubitId(*j as u32)).unwrap();
    }
}

/// Apply the mixer unitary.
///
/// For each qubit, apply RX(2β).
fn apply_mixer_unitary(circuit: &mut Circuit, n_qubits: usize, beta: f64) {
    let angle = 2.0 * beta;
    for q in 0..n_qubits {
        circuit.rx(angle, QubitId(q as u32)).unwrap();
    }
}

/// Generate a QAOA circuit without measurements (for expectation value calculation).
pub fn qaoa_circuit_no_measure(graph: &Graph, gamma: &[f64], beta: &[f64]) -> Circuit {
    assert_eq!(gamma.len(), beta.len());
    let p = gamma.len();
    let n = graph.n_nodes;

    let mut circuit = Circuit::with_size("qaoa", n as u32, 0);

    // Initialize |+⟩^n
    for q in 0..n {
        circuit.h(QubitId(q as u32)).unwrap();
    }

    // Apply p layers
    for layer in 0..p {
        apply_cost_unitary(&mut circuit, graph, gamma[layer]);
        apply_mixer_unitary(&mut circuit, n, beta[layer]);
    }

    circuit
}

/// Calculate the optimal initial parameters for QAOA.
///
/// Returns (gamma, beta) initialized to reasonable starting values.
/// Based on heuristics from the literature.
pub fn initial_parameters(p: usize) -> (Vec<f64>, Vec<f64>) {
    // Interpolation-based initialization
    // gamma starts small and increases, beta starts large and decreases
    let gamma: Vec<f64> = (0..p).map(|i| PI / 4.0 * (i + 1) as f64 / p as f64).collect();
    let beta: Vec<f64> = (0..p).map(|i| PI / 4.0 * (p - i) as f64 / p as f64).collect();
    (gamma, beta)
}

/// Calculate the number of QAOA parameters.
pub fn num_parameters(p: usize) -> usize {
    2 * p // p gamma values + p beta values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qaoa_circuit() {
        let graph = Graph::square_4();
        let gamma = vec![0.5];
        let beta = vec![0.3];

        let circuit = qaoa_circuit(&graph, &gamma, &beta);

        assert_eq!(circuit.num_qubits(), 4);
        assert_eq!(circuit.num_clbits(), 4);
        assert!(circuit.depth() > 0);
    }

    #[test]
    fn test_qaoa_multi_layer() {
        let graph = Graph::square_4();
        let gamma = vec![0.1, 0.2, 0.3];
        let beta = vec![0.3, 0.2, 0.1];

        let circuit = qaoa_circuit(&graph, &gamma, &beta);

        assert_eq!(circuit.num_qubits(), 4);
    }

    #[test]
    fn test_initial_parameters() {
        let (gamma, beta) = initial_parameters(3);

        assert_eq!(gamma.len(), 3);
        assert_eq!(beta.len(), 3);

        // Gamma should be increasing
        assert!(gamma[0] < gamma[1]);
        assert!(gamma[1] < gamma[2]);

        // Beta should be decreasing
        assert!(beta[0] > beta[1]);
        assert!(beta[1] > beta[2]);
    }

    #[test]
    fn test_num_parameters() {
        assert_eq!(num_parameters(1), 2);
        assert_eq!(num_parameters(3), 6);
    }
}
