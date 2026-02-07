#!/bin/bash
#SBATCH --job-name=arvak_vqe_classical
#SBATCH --account=project_462000xxx
#SBATCH --partition=small-g
#SBATCH --time=01:00:00
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --gpus-per-node=1
#SBATCH --output=classical_%j.out
#SBATCH --error=classical_%j.err

# ╔════════════════════════════════════════════════════════════════════════════╗
# ║  LUMI-G Classical Job Script                                                ║
# ║  Runs classical optimizer for VQE on AMD MI250X GPU                         ║
# ╚════════════════════════════════════════════════════════════════════════════╝
#
# This script runs on LUMI-G (GPU partition) and performs:
# - Classical optimization (COBYLA, SPSA, etc.)
# - Parameter updates for variational circuits
# - Result analysis and visualization
#
# Usage:
#   sbatch --export=CONFIG_FILE=vqe_config.yaml classical_job.sh
#
# For CPU-only execution, use partition=small instead of small-g

echo "╔════════════════════════════════════════════════════════════════════════════╗"
echo "║  LUMI-G Arvak Classical Optimizer                                             ║"
echo "╠════════════════════════════════════════════════════════════════════════════╣"
echo "║  Job ID: $SLURM_JOB_ID"
echo "║  Node:   $SLURMD_NODENAME"
echo "║  GPUs:   $SLURM_GPUS_ON_NODE"
echo "║  Time:   $(date)"
echo "╚════════════════════════════════════════════════════════════════════════════╝"

# Load modules
module load LUMI/23.09
module load cray-python/3.10.10
module load rocm/5.6.1

# Set paths
ARVAK_DIR="${ARVAK_DIR:-$HOME/arvak}"
WORK_DIR="${SLURM_SUBMIT_DIR:-$(pwd)}"

# Configuration
CONFIG_FILE="${CONFIG_FILE:-$WORK_DIR/vqe_config.yaml}"
RESULTS_DIR="${RESULTS_DIR:-$WORK_DIR/results}"

echo ""
echo "Configuration:"
echo "  ARVAK_DIR:     $ARVAK_DIR"
echo "  CONFIG_FILE: $CONFIG_FILE"
echo "  RESULTS_DIR: $RESULTS_DIR"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# GPU info
echo "GPU Information:"
rocm-smi --showid --showbus --showtemp 2>/dev/null || echo "  ROCm not available"
echo ""

# Check if we're in hybrid mode (coordinating with quantum jobs)
HYBRID_MODE="${HYBRID_MODE:-false}"

if [[ "$HYBRID_MODE" == "true" ]]; then
    echo "Running in HYBRID mode (classical + quantum coordination)"
    echo ""

    # Run the VQE coordinator
    "$ARVAK_DIR/target/release/lumi_vqe" \
        --backend lumi \
        --max-iterations "${MAX_ITERATIONS:-50}" \
        --shots "${SHOTS:-1000}" \
        --output "$RESULTS_DIR" \
        --slurm \
        --verbose

else
    echo "Running in STANDALONE mode (simulator only)"
    echo ""

    # Run VQE with local simulator
    "$ARVAK_DIR/target/release/lumi_vqe" \
        --backend sim \
        --max-iterations "${MAX_ITERATIONS:-50}" \
        --shots "${SHOTS:-1000}" \
        --output "$RESULTS_DIR" \
        --verbose
fi

EXIT_CODE=$?

echo ""
echo "VQE optimization completed with exit code: $EXIT_CODE"

# Show results summary
if [[ -f "$RESULTS_DIR/vqe_result.json" ]]; then
    echo ""
    echo "Results summary:"
    python3 -c "
import json
with open('$RESULTS_DIR/vqe_result.json') as f:
    r = json.load(f)
print(f\"  Bond distance: {r['bond_distance']:.3f} Å\")
print(f\"  Final energy:  {r['final_energy']:.6f} Ha\")
print(f\"  Exact energy:  {r['exact_energy']:.6f} Ha\")
print(f\"  Error:         {r['error']*1000:.4f} mHa\")
print(f\"  Iterations:    {len(r['iterations'])}\")
print(f\"  Total shots:   {r['total_shots']}\")
"
fi

# Generate visualization if matplotlib is available
python3 << 'PYTHON_SCRIPT'
import json
import os

results_dir = os.environ.get('RESULTS_DIR', 'results')
result_file = os.path.join(results_dir, 'vqe_result.json')

if not os.path.exists(result_file):
    print("No result file found, skipping visualization")
    exit(0)

try:
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt

    with open(result_file) as f:
        data = json.load(f)

    iterations = [r['iteration'] for r in data['iterations']]
    energies = [r['energy'] for r in data['iterations']]
    exact = data['exact_energy']

    fig, ax = plt.subplots(figsize=(10, 6))
    ax.plot(iterations, energies, 'b-o', label='VQE Energy', markersize=4)
    ax.axhline(y=exact, color='r', linestyle='--', label=f'Exact: {exact:.4f} Ha')

    ax.set_xlabel('Iteration')
    ax.set_ylabel('Energy (Hartree)')
    ax.set_title(f'VQE Convergence for H₂ (r = {data["bond_distance"]:.3f} Å)')
    ax.legend()
    ax.grid(True, alpha=0.3)

    plot_file = os.path.join(results_dir, 'convergence.png')
    plt.savefig(plot_file, dpi=150, bbox_inches='tight')
    print(f"Convergence plot saved to: {plot_file}")

except ImportError:
    print("matplotlib not available, skipping visualization")
except Exception as e:
    print(f"Visualization failed: {e}")
PYTHON_SCRIPT

exit $EXIT_CODE
