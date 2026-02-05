//! Molecular Hamiltonians for VQE.
//!
//! These Hamiltonians are pre-computed using the Jordan-Wigner transformation
//! from second-quantized fermionic operators to qubit operators.

use super::hamiltonian::{Pauli, PauliHamiltonian, PauliTerm};

/// H2 molecule Hamiltonian at equilibrium bond distance (0.735 Angstrom).
///
/// This is a 2-qubit Hamiltonian obtained from the Jordan-Wigner transformation
/// of the H2 molecule in the minimal STO-3G basis.
///
/// Exact ground state energy: -1.137 Hartree
///
/// H = g0 I + g1 Z0 + g2 Z1 + g3 Z0Z1 + g4 X0X1 + g5 Y0Y1
///
/// Coefficients from Qiskit Nature / PySCF:
/// - g0 = -1.0523
/// - g1 = 0.3979
/// - g2 = -0.3979
/// - g3 = -0.0112
/// - g4 = 0.1809
/// - g5 = 0.1809
pub fn h2_hamiltonian() -> PauliHamiltonian {
    PauliHamiltonian::new(vec![
        PauliTerm::identity(-1.0523),
        PauliTerm::z(0.3979, 0),
        PauliTerm::z(-0.3979, 1),
        PauliTerm::zz(-0.0112, 0, 1),
        PauliTerm::xx(0.1809, 0, 1),
        PauliTerm::yy(0.1809, 0, 1),
    ])
}

/// H2 molecule Hamiltonian in 4-qubit encoding.
///
/// This is the full 4-qubit representation using the Jordan-Wigner transformation
/// with all spin-orbitals. More realistic but requires more qubits.
///
/// Exact ground state energy: -1.137 Hartree
pub fn h2_hamiltonian_4q() -> PauliHamiltonian {
    // Simplified 4-qubit encoding coefficients
    PauliHamiltonian::new(vec![
        PauliTerm::identity(-0.8105),
        PauliTerm::z(0.1721, 0),
        PauliTerm::z(0.1721, 1),
        PauliTerm::z(-0.2234, 2),
        PauliTerm::z(-0.2234, 3),
        PauliTerm::zz(0.1209, 0, 1),
        PauliTerm::zz(0.1686, 0, 2),
        PauliTerm::zz(0.1205, 0, 3),
        PauliTerm::zz(0.1205, 1, 2),
        PauliTerm::zz(0.1686, 1, 3),
        PauliTerm::zz(0.1744, 2, 3),
        PauliTerm::new(0.0453, vec![(0, Pauli::X), (1, Pauli::X), (2, Pauli::Y), (3, Pauli::Y)]),
        PauliTerm::new(0.0453, vec![(0, Pauli::Y), (1, Pauli::Y), (2, Pauli::X), (3, Pauli::X)]),
        PauliTerm::new(-0.0453, vec![(0, Pauli::X), (1, Pauli::Y), (2, Pauli::Y), (3, Pauli::X)]),
        PauliTerm::new(-0.0453, vec![(0, Pauli::Y), (1, Pauli::X), (2, Pauli::X), (3, Pauli::Y)]),
    ])
}

/// LiH molecule Hamiltonian (simplified 4-qubit version).
///
/// Lithium Hydride at equilibrium geometry.
/// This is an approximation for demo purposes.
///
/// Exact ground state energy: approximately -7.882 Hartree
pub fn lih_hamiltonian() -> PauliHamiltonian {
    // Simplified LiH coefficients (reduced from full representation)
    PauliHamiltonian::new(vec![
        PauliTerm::identity(-7.4983),
        PauliTerm::z(0.1122, 0),
        PauliTerm::z(0.1122, 1),
        PauliTerm::z(-0.1347, 2),
        PauliTerm::z(-0.1347, 3),
        PauliTerm::zz(0.0892, 0, 1),
        PauliTerm::zz(0.1104, 0, 2),
        PauliTerm::zz(0.0983, 0, 3),
        PauliTerm::zz(0.0983, 1, 2),
        PauliTerm::zz(0.1104, 1, 3),
        PauliTerm::zz(0.1205, 2, 3),
        PauliTerm::xx(0.0312, 0, 1),
        PauliTerm::yy(0.0312, 0, 1),
        PauliTerm::xx(0.0245, 2, 3),
        PauliTerm::yy(0.0245, 2, 3),
    ])
}

/// Get the exact ground state energy for a known molecule.
pub fn exact_ground_state_energy(molecule: &str) -> Option<f64> {
    match molecule.to_lowercase().as_str() {
        "h2" => Some(-1.137),
        "lih" => Some(-7.882),
        "beh2" => Some(-15.835),
        "h2o" => Some(-75.012),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h2_hamiltonian() {
        let h = h2_hamiltonian();
        assert_eq!(h.num_qubits(), 2);
        assert_eq!(h.num_terms(), 6);

        // Check identity coefficient
        assert!((h.identity_coefficient() - (-1.0523)).abs() < 1e-4);
    }

    #[test]
    fn test_h2_4q_hamiltonian() {
        let h = h2_hamiltonian_4q();
        assert_eq!(h.num_qubits(), 4);
        assert!(h.num_terms() > 10);
    }

    #[test]
    fn test_lih_hamiltonian() {
        let h = lih_hamiltonian();
        assert_eq!(h.num_qubits(), 4);
    }

    #[test]
    fn test_exact_energies() {
        assert_eq!(exact_ground_state_energy("h2"), Some(-1.137));
        assert_eq!(exact_ground_state_energy("H2"), Some(-1.137));
        assert_eq!(exact_ground_state_energy("unknown"), None);
    }
}
