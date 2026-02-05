#!/usr/bin/env python3
"""
VQE Results Visualization

This script generates plots from VQE results:
- Convergence plot (energy vs iteration)
- Bond distance scan (potential energy surface)
- Parameter evolution

Usage:
    python plot_results.py results/vqe_result.json
    python plot_results.py results/bond_scan.json --scan
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import matplotlib.pyplot as plt
    import numpy as np
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    print("Warning: matplotlib not available, text output only")


def plot_convergence(data: dict, output_dir: Path):
    """Plot VQE convergence (energy vs iteration)."""
    if not HAS_MATPLOTLIB:
        return

    iterations = [r['iteration'] for r in data['iterations']]
    energies = [r['energy'] for r in data['iterations']]
    exact = data['exact_energy']

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

    # Energy convergence
    ax1.plot(iterations, energies, 'b-o', label='VQE Energy', markersize=4)
    ax1.axhline(y=exact, color='r', linestyle='--', label=f'Exact: {exact:.6f} Ha')
    ax1.set_xlabel('Iteration')
    ax1.set_ylabel('Energy (Hartree)')
    ax1.set_title(f'VQE Convergence for H₂ (r = {data["bond_distance"]:.3f} Å)')
    ax1.legend()
    ax1.grid(True, alpha=0.3)

    # Error convergence (log scale)
    errors = [abs(e - exact) * 1000 for e in energies]  # mHa
    ax2.semilogy(iterations, errors, 'g-s', markersize=4)
    ax2.axhline(y=1.6, color='orange', linestyle='--', label='Chemical accuracy (1.6 mHa)')
    ax2.set_xlabel('Iteration')
    ax2.set_ylabel('Error (mHa)')
    ax2.set_title('Convergence to Exact Energy')
    ax2.legend()
    ax2.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = output_dir / 'convergence.png'
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    print(f"Convergence plot saved to: {output_file}")
    plt.close()


def plot_bond_scan(data: list, output_dir: Path):
    """Plot potential energy surface (energy vs bond distance)."""
    if not HAS_MATPLOTLIB:
        return

    distances = [r['bond_distance'] for r in data]
    vqe_energies = [r['final_energy'] for r in data]
    exact_energies = [r['exact_energy'] for r in data]
    errors = [r['error'] * 1000 for r in data]  # mHa

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

    # Potential energy surface
    ax1.plot(distances, vqe_energies, 'b-o', label='VQE', markersize=6)
    ax1.plot(distances, exact_energies, 'r--', label='Exact', linewidth=2)
    ax1.set_xlabel('Bond Distance (Å)')
    ax1.set_ylabel('Energy (Hartree)')
    ax1.set_title('H₂ Potential Energy Surface')
    ax1.legend()
    ax1.grid(True, alpha=0.3)

    # Find and mark equilibrium
    min_idx = np.argmin(exact_energies)
    ax1.axvline(x=distances[min_idx], color='gray', linestyle=':', alpha=0.5)
    ax1.annotate(f'r_eq = {distances[min_idx]:.2f} Å',
                 xy=(distances[min_idx], exact_energies[min_idx]),
                 xytext=(distances[min_idx] + 0.3, exact_energies[min_idx] + 0.05),
                 arrowprops=dict(arrowstyle='->', color='gray'))

    # Error vs distance
    ax2.bar(distances, errors, width=0.08, color='green', alpha=0.7)
    ax2.axhline(y=1.6, color='orange', linestyle='--', label='Chemical accuracy')
    ax2.set_xlabel('Bond Distance (Å)')
    ax2.set_ylabel('Error (mHa)')
    ax2.set_title('VQE Error vs Bond Distance')
    ax2.legend()
    ax2.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    output_file = output_dir / 'bond_scan.png'
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    print(f"Bond scan plot saved to: {output_file}")
    plt.close()


def print_summary(data, is_scan=False):
    """Print text summary of results."""
    print()
    print("=" * 70)

    if is_scan:
        print("H₂ Bond Distance Scan Results")
        print("=" * 70)
        print(f"{'r (Å)':<10} {'VQE (Ha)':<15} {'Exact (Ha)':<15} {'Error (mHa)':<15}")
        print("-" * 70)
        for r in data:
            print(f"{r['bond_distance']:<10.3f} {r['final_energy']:<15.6f} "
                  f"{r['exact_energy']:<15.6f} {r['error']*1000:<15.4f}")
    else:
        print("VQE Result Summary")
        print("=" * 70)
        print(f"Bond distance:   {data['bond_distance']:.3f} Å")
        print(f"Final energy:    {data['final_energy']:.6f} Ha")
        print(f"Exact energy:    {data['exact_energy']:.6f} Ha")
        print(f"Error:           {data['error']*1000:.4f} mHa")
        print(f"Iterations:      {len(data['iterations'])}")
        print(f"Total shots:     {data['total_shots']}")
        print(f"Backend:         {data['backend']}")

        # Check chemical accuracy
        if data['error'] * 1000 < 1.6:
            print()
            print("✓ Chemical accuracy achieved! (< 1.6 mHa)")
        else:
            print()
            print(f"✗ Above chemical accuracy ({data['error']*1000:.2f} mHa > 1.6 mHa)")

    print("=" * 70)
    print()


def main():
    parser = argparse.ArgumentParser(description='Plot VQE results')
    parser.add_argument('input_file', type=Path, help='JSON result file')
    parser.add_argument('--scan', action='store_true', help='Bond scan mode')
    parser.add_argument('--output', type=Path, default=None, help='Output directory')
    args = parser.parse_args()

    # Read data
    if not args.input_file.exists():
        print(f"Error: File not found: {args.input_file}")
        sys.exit(1)

    with open(args.input_file) as f:
        data = json.load(f)

    # Set output directory
    output_dir = args.output or args.input_file.parent
    output_dir.mkdir(parents=True, exist_ok=True)

    # Print summary
    print_summary(data, is_scan=args.scan)

    # Generate plots
    if args.scan:
        plot_bond_scan(data, output_dir)
    else:
        plot_convergence(data, output_dir)

    print("Done!")


if __name__ == '__main__':
    main()
