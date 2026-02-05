//! VQE (Variational Quantum Eigensolver) runner.
//!
//! VQE is a hybrid classical-quantum algorithm for finding ground state
//! energies of quantum systems.

use crate::circuits::vqe::{num_parameters, two_local_ansatz};
use crate::optimizers::{Cobyla, Optimizer};
use crate::problems::{Pauli, PauliHamiltonian};

/// Result of a VQE run.
#[derive(Debug, Clone)]
pub struct VqeResult {
    /// Optimal energy found.
    pub optimal_energy: f64,
    /// Optimal parameters.
    pub optimal_params: Vec<f64>,
    /// Number of iterations.
    pub iterations: usize,
    /// Number of circuit evaluations.
    pub circuit_evaluations: usize,
    /// Energy history during optimization.
    pub energy_history: Vec<f64>,
    /// Whether optimization converged.
    pub converged: bool,
}

/// VQE runner configuration.
pub struct VqeRunner {
    /// The Hamiltonian to minimize.
    pub hamiltonian: PauliHamiltonian,
    /// Number of qubits.
    pub n_qubits: usize,
    /// Number of ansatz repetitions.
    pub reps: usize,
    /// Number of measurement shots per evaluation.
    pub shots: u32,
    /// Maximum optimization iterations.
    pub maxiter: usize,
}

impl VqeRunner {
    /// Create a new VQE runner.
    pub fn new(hamiltonian: PauliHamiltonian) -> Self {
        let n_qubits = hamiltonian.num_qubits();
        Self {
            hamiltonian,
            n_qubits,
            reps: 2,
            shots: 1024,
            maxiter: 100,
        }
    }

    /// Set the number of ansatz repetitions.
    pub fn with_reps(mut self, reps: usize) -> Self {
        self.reps = reps;
        self
    }

    /// Set the number of shots.
    pub fn with_shots(mut self, shots: u32) -> Self {
        self.shots = shots;
        self
    }

    /// Set maximum iterations.
    pub fn with_maxiter(mut self, maxiter: usize) -> Self {
        self.maxiter = maxiter;
        self
    }

    /// Run VQE with random initial parameters.
    pub fn run(&self) -> VqeResult {
        let num_params = num_parameters("two_local", self.n_qubits, self.reps);

        // Initialize parameters randomly
        let mut seed: u64 = 42;
        let initial_params: Vec<f64> = (0..num_params)
            .map(|_| {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                (seed as f64 / u64::MAX as f64) * std::f64::consts::PI - std::f64::consts::PI / 2.0
            })
            .collect();

        self.run_with_params(initial_params)
    }

    /// Run VQE with specified initial parameters.
    pub fn run_with_params(&self, initial_params: Vec<f64>) -> VqeResult {
        let mut circuit_evaluations = 0;

        // Create optimizer
        let optimizer = Cobyla::new()
            .with_maxiter(self.maxiter)
            .with_tol(1e-6);

        // Objective function: evaluate energy
        let hamiltonian = &self.hamiltonian;
        let n_qubits = self.n_qubits;
        let reps = self.reps;
        let shots = self.shots;

        let objective = |params: &[f64]| -> f64 {
            circuit_evaluations += 1;
            evaluate_energy(hamiltonian, n_qubits, reps, params, shots)
        };

        let result = optimizer.minimize(objective, initial_params);

        VqeResult {
            optimal_energy: result.optimal_value,
            optimal_params: result.optimal_params,
            iterations: result.num_iterations,
            circuit_evaluations: result.num_evaluations,
            energy_history: result.history,
            converged: result.converged,
        }
    }

    /// Get the number of parameters needed.
    pub fn num_parameters(&self) -> usize {
        num_parameters("two_local", self.n_qubits, self.reps)
    }
}

/// Evaluate the energy expectation value for given parameters.
///
/// This simulates the quantum circuit execution and measurement.
/// In a real system, this would submit a job to a quantum backend.
fn evaluate_energy(
    hamiltonian: &PauliHamiltonian,
    n_qubits: usize,
    reps: usize,
    params: &[f64],
    shots: u32,
) -> f64 {
    // Build the ansatz circuit
    let circuit = two_local_ansatz(n_qubits, reps, params);

    // Simulate the statevector (simplified)
    let statevector = simulate_statevector(&circuit, n_qubits);

    // Calculate expectation value
    expectation_value(hamiltonian, &statevector)
}

/// Simplified statevector simulation.
///
/// This is a basic simulator for demo purposes.
/// In production, use a proper simulator or quantum hardware.
fn simulate_statevector(circuit: &hiq_ir::Circuit, n_qubits: usize) -> Vec<num_complex::Complex64> {
    use num_complex::Complex64;

    let dim = 1 << n_qubits;
    let mut state = vec![Complex64::new(0.0, 0.0); dim];
    state[0] = Complex64::new(1.0, 0.0); // |0...0⟩

    // Apply gates from the circuit DAG
    for (_, instr) in circuit.dag().topological_ops() {
        if let hiq_ir::instruction::InstructionKind::Gate(gate) = &instr.kind {
            let qubits: Vec<usize> = instr.qubits.iter().map(|q| q.0 as usize).collect();

            match &gate.kind {
                hiq_ir::gate::GateKind::Standard(std_gate) => {
                    apply_gate(&mut state, std_gate, &qubits);
                }
                _ => {}
            }
        }
    }

    state
}

/// Apply a standard gate to the statevector.
fn apply_gate(
    state: &mut [num_complex::Complex64],
    gate: &hiq_ir::gate::StandardGate,
    qubits: &[usize],
) {
    use hiq_ir::gate::StandardGate;
    use num_complex::Complex64;

    #[allow(unused_variables)]
    let n = (state.len() as f64).log2() as usize;

    match gate {
        StandardGate::H => {
            let q = qubits[0];
            let h = std::f64::consts::FRAC_1_SQRT_2;
            for i in 0..state.len() {
                if (i >> q) & 1 == 0 {
                    let j = i | (1 << q);
                    let a = state[i];
                    let b = state[j];
                    state[i] = Complex64::new(h, 0.0) * (a + b);
                    state[j] = Complex64::new(h, 0.0) * (a - b);
                }
            }
        }
        StandardGate::X => {
            let q = qubits[0];
            for i in 0..state.len() {
                if (i >> q) & 1 == 0 {
                    let j = i | (1 << q);
                    state.swap(i, j);
                }
            }
        }
        StandardGate::Y => {
            let q = qubits[0];
            for i in 0..state.len() {
                if (i >> q) & 1 == 0 {
                    let j = i | (1 << q);
                    let a = state[i];
                    let b = state[j];
                    state[i] = Complex64::new(0.0, 1.0) * b;
                    state[j] = Complex64::new(0.0, -1.0) * a;
                }
            }
        }
        StandardGate::Z => {
            let q = qubits[0];
            for i in 0..state.len() {
                if (i >> q) & 1 == 1 {
                    state[i] = -state[i];
                }
            }
        }
        StandardGate::Ry(param) => {
            if let Some(theta) = param.as_f64() {
                let q = qubits[0];
                let c = (theta / 2.0).cos();
                let s = (theta / 2.0).sin();
                for i in 0..state.len() {
                    if (i >> q) & 1 == 0 {
                        let j = i | (1 << q);
                        let a = state[i];
                        let b = state[j];
                        state[i] = Complex64::new(c, 0.0) * a - Complex64::new(s, 0.0) * b;
                        state[j] = Complex64::new(s, 0.0) * a + Complex64::new(c, 0.0) * b;
                    }
                }
            }
        }
        StandardGate::Rz(param) => {
            if let Some(theta) = param.as_f64() {
                let q = qubits[0];
                let phase0 = Complex64::new((-theta / 2.0).cos(), (-theta / 2.0).sin());
                let phase1 = Complex64::new((theta / 2.0).cos(), (theta / 2.0).sin());
                for i in 0..state.len() {
                    if (i >> q) & 1 == 0 {
                        state[i] = phase0 * state[i];
                    } else {
                        state[i] = phase1 * state[i];
                    }
                }
            }
        }
        StandardGate::Rx(param) => {
            if let Some(theta) = param.as_f64() {
                let q = qubits[0];
                let c = (theta / 2.0).cos();
                let s = (theta / 2.0).sin();
                for i in 0..state.len() {
                    if (i >> q) & 1 == 0 {
                        let j = i | (1 << q);
                        let a = state[i];
                        let b = state[j];
                        state[i] = Complex64::new(c, 0.0) * a - Complex64::new(0.0, s) * b;
                        state[j] = Complex64::new(0.0, -s) * a + Complex64::new(c, 0.0) * b;
                    }
                }
            }
        }
        StandardGate::CX => {
            let control = qubits[0];
            let target = qubits[1];
            for i in 0..state.len() {
                if (i >> control) & 1 == 1 && (i >> target) & 1 == 0 {
                    let j = i | (1 << target);
                    state.swap(i, j);
                }
            }
        }
        StandardGate::CZ => {
            let q0 = qubits[0];
            let q1 = qubits[1];
            for i in 0..state.len() {
                if (i >> q0) & 1 == 1 && (i >> q1) & 1 == 1 {
                    state[i] = -state[i];
                }
            }
        }
        _ => {
            // Other gates not implemented for this demo
        }
    }
}

/// Calculate the expectation value of a Hamiltonian.
fn expectation_value(hamiltonian: &PauliHamiltonian, statevector: &[num_complex::Complex64]) -> f64 {
    use num_complex::Complex64;

    let n = (statevector.len() as f64).log2() as usize;
    let mut energy = 0.0;

    for term in &hamiltonian.terms {
        let mut term_value = Complex64::new(0.0, 0.0);

        // Calculate <ψ|P|ψ> for this Pauli term
        for (i, &amplitude) in statevector.iter().enumerate() {
            // Apply Pauli operators and compute inner product
            let (j, phase) = apply_pauli_string(i, &term.operators, n);
            term_value += amplitude.conj() * phase * statevector[j];
        }

        energy += term.coefficient * term_value.re;
    }

    energy
}

/// Apply a Pauli string to a basis state index.
/// Returns the new index and accumulated phase.
fn apply_pauli_string(
    index: usize,
    operators: &[(usize, Pauli)],
    n_qubits: usize,
) -> (usize, num_complex::Complex64) {
    use num_complex::Complex64;

    let mut new_index = index;
    let mut phase = Complex64::new(1.0, 0.0);

    for &(qubit, pauli) in operators {
        let bit = (index >> qubit) & 1;

        match pauli {
            Pauli::I => {}
            Pauli::X => {
                new_index ^= 1 << qubit;
            }
            Pauli::Y => {
                new_index ^= 1 << qubit;
                if bit == 0 {
                    phase *= Complex64::new(0.0, 1.0);
                } else {
                    phase *= Complex64::new(0.0, -1.0);
                }
            }
            Pauli::Z => {
                if bit == 1 {
                    phase *= Complex64::new(-1.0, 0.0);
                }
            }
        }
    }

    (new_index, phase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::{h2_hamiltonian, PauliTerm};

    #[test]
    fn test_vqe_runner_creation() {
        let h = h2_hamiltonian();
        let runner = VqeRunner::new(h).with_reps(2).with_maxiter(10);

        assert_eq!(runner.n_qubits, 2);
        assert_eq!(runner.reps, 2);
        assert_eq!(runner.maxiter, 10);
    }

    #[test]
    fn test_vqe_simple_run() {
        let h = h2_hamiltonian();
        let runner = VqeRunner::new(h).with_reps(1).with_maxiter(20);

        let result = runner.run();

        // H2 ground state energy is about -1.137
        // With limited iterations, we should at least get negative energy
        assert!(result.optimal_energy < 0.0);
    }

    #[test]
    fn test_expectation_value() {
        use num_complex::Complex64;

        // Test with |00⟩ state and Z0 operator
        let state = vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        ];

        let h = PauliHamiltonian::new(vec![PauliTerm::z(1.0, 0)]);

        let energy = expectation_value(&h, &state);
        assert!((energy - 1.0).abs() < 1e-10); // <00|Z0|00> = 1
    }
}
