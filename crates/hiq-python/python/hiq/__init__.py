"""HIQ: Rust-native quantum compilation platform.

This module provides Python bindings for the HIQ quantum circuit
builder and compilation framework.

Example:
    >>> import hiq
    >>> qc = hiq.Circuit("bell", num_qubits=2)
    >>> qc.h(0).cx(0, 1)
    >>> print(hiq.to_qasm(qc))
"""

# Re-export everything from the native extension
from hiq.hiq import (
    # Core types
    Circuit,
    QubitId,
    ClbitId,
    # Compilation types
    Layout,
    CouplingMap,
    BasisGates,
    PropertySet,
    # QASM I/O
    from_qasm,
    to_qasm,
)

__all__ = [
    "Circuit",
    "QubitId",
    "ClbitId",
    "Layout",
    "CouplingMap",
    "BasisGates",
    "PropertySet",
    "from_qasm",
    "to_qasm",
]
