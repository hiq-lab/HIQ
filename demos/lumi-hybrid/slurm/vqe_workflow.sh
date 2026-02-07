#!/bin/bash
# ╔════════════════════════════════════════════════════════════════════════════╗
# ║  LUMI Hybrid VQE Workflow                                                   ║
# ║  Coordinates classical (LUMI-G) and quantum (LUMI-Q) jobs                   ║
# ╚════════════════════════════════════════════════════════════════════════════╝
#
# This script orchestrates the full VQE workflow on LUMI:
# 1. Submit classical optimizer job to LUMI-G/LUMI-C
# 2. Classical job spawns quantum evaluation jobs to LUMI-Q
# 3. Quantum results are collected and fed back to optimizer
# 4. Repeat until convergence
#
# Usage:
#   ./vqe_workflow.sh [--bond-distance 0.735] [--max-iterations 50] [--shots 1000]
#
# Prerequisites:
#   - LUMI account with access to q_fiqci (quantum) and small-g (GPU) partitions
#   - Arvak compiled and installed in $HOME/arvak
#   - HELMI_TOKEN environment variable set

set -e

# ═══════════════════════════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════════════════════════

# Default values
BOND_DISTANCE="${BOND_DISTANCE:-0.735}"
MAX_ITERATIONS="${MAX_ITERATIONS:-50}"
SHOTS="${SHOTS:-1000}"
ACCOUNT="${ACCOUNT:-project_462000xxx}"
ARVAK_DIR="${ARVAK_DIR:-$HOME/arvak}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --bond-distance)
            BOND_DISTANCE="$2"
            shift 2
            ;;
        --max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        --shots)
            SHOTS="$2"
            shift 2
            ;;
        --account)
            ACCOUNT="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --bond-distance R    H-H bond distance in Angstroms (default: 0.735)"
            echo "  --max-iterations N   Maximum VQE iterations (default: 50)"
            echo "  --shots N            Shots per circuit evaluation (default: 1000)"
            echo "  --account PROJECT    LUMI project account"
            echo "  --help               Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# ═══════════════════════════════════════════════════════════════════════════════
# Setup
# ═══════════════════════════════════════════════════════════════════════════════

echo "╔════════════════════════════════════════════════════════════════════════════╗"
echo "║        LUMI Hybrid VQE Workflow for H₂ Ground State                         ║"
echo "╠════════════════════════════════════════════════════════════════════════════╣"
echo "║  Bond distance:   $BOND_DISTANCE Å"
echo "║  Max iterations:  $MAX_ITERATIONS"
echo "║  Shots/circuit:   $SHOTS"
echo "║  Account:         $ACCOUNT"
echo "╚════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Create working directory
WORK_DIR="vqe_run_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo "Working directory: $(pwd)"
echo ""

# Check authentication
if [[ -z "$HELMI_TOKEN" && -z "$IQM_TOKEN" ]]; then
    echo "⚠️  WARNING: No quantum authentication token found"
    echo "   Set HELMI_TOKEN for LUMI-Q access"
    echo "   Workflow will use simulator for quantum evaluations"
    echo ""
    USE_SIMULATOR=true
else
    echo "✓ Authentication token found"
    USE_SIMULATOR=false
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Create workflow configuration
# ═══════════════════════════════════════════════════════════════════════════════

cat > vqe_config.yaml << EOF
# VQE Workflow Configuration
# Generated: $(date)

workflow:
  name: h2_vqe
  type: hybrid_quantum_classical

molecule:
  name: H2
  bond_distance: $BOND_DISTANCE
  basis: STO-3G

vqe:
  ansatz: uccsd
  optimizer: cobyla
  max_iterations: $MAX_ITERATIONS
  convergence_threshold: 1.0e-4

quantum:
  shots: $SHOTS
  backend: ${USE_SIMULATOR:+sim}${USE_SIMULATOR:-iqm}
  partition: q_fiqci

classical:
  partition: small-g
  gpus: 1

slurm:
  account: $ACCOUNT
  time_quantum: "00:30:00"
  time_classical: "01:00:00"
EOF

echo "Configuration written to vqe_config.yaml"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# Submit workflow
# ═══════════════════════════════════════════════════════════════════════════════

echo "Submitting VQE workflow..."
echo ""

# Option 1: Full hybrid mode (requires quantum access)
if [[ "$USE_SIMULATOR" == "false" ]]; then
    echo "Mode: HYBRID (LUMI-G + LUMI-Q)"
    echo ""

    # Submit coordinator job to LUMI-G
    CLASSICAL_JOB=$(sbatch \
        --account="$ACCOUNT" \
        --partition=small-g \
        --time=01:00:00 \
        --nodes=1 \
        --gpus-per-node=1 \
        --export=ALL,ARVAK_DIR="$ARVAK_DIR",HYBRID_MODE=true,MAX_ITERATIONS="$MAX_ITERATIONS",SHOTS="$SHOTS",RESULTS_DIR="$(pwd)/results" \
        --output=classical_%j.out \
        --error=classical_%j.err \
        "$ARVAK_DIR/demos/lumi-hybrid/slurm/classical_job.sh" \
        | awk '{print $4}')

    echo "Submitted classical coordinator: Job ID $CLASSICAL_JOB"

# Option 2: Simulator-only mode
else
    echo "Mode: SIMULATOR (local quantum simulation)"
    echo ""

    # Submit simulator job to LUMI-C (CPU partition)
    CLASSICAL_JOB=$(sbatch \
        --account="$ACCOUNT" \
        --partition=small \
        --time=00:30:00 \
        --nodes=1 \
        --cpus-per-task=8 \
        --export=ALL,ARVAK_DIR="$ARVAK_DIR",HYBRID_MODE=false,MAX_ITERATIONS="$MAX_ITERATIONS",SHOTS="$SHOTS",RESULTS_DIR="$(pwd)/results" \
        --output=classical_%j.out \
        --error=classical_%j.err \
        "$ARVAK_DIR/demos/lumi-hybrid/slurm/classical_job.sh" \
        | awk '{print $4}')

    echo "Submitted simulator job: Job ID $CLASSICAL_JOB"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# Monitor progress
# ═══════════════════════════════════════════════════════════════════════════════

echo "To monitor progress:"
echo "  squeue -u \$USER"
echo "  tail -f classical_${CLASSICAL_JOB}.out"
echo ""
echo "Results will be written to: $(pwd)/results/"
echo ""

# Create convenience script for checking results
cat > check_results.sh << 'EOF'
#!/bin/bash
if [[ -f results/vqe_result.json ]]; then
    python3 -c "
import json
with open('results/vqe_result.json') as f:
    r = json.load(f)
print('═' * 60)
print('VQE Results for H₂')
print('═' * 60)
print(f\"Bond distance: {r['bond_distance']:.3f} Å\")
print(f\"Final energy:  {r['final_energy']:.6f} Ha\")
print(f\"Exact energy:  {r['exact_energy']:.6f} Ha\")
print(f\"Error:         {r['error']*1000:.4f} mHa\")
print(f\"Iterations:    {len(r['iterations'])}\")
print(f\"Total shots:   {r['total_shots']}\")
print('═' * 60)
"
else
    echo "Results not yet available"
fi
EOF
chmod +x check_results.sh

echo "Run ./check_results.sh to view results when complete"
echo ""

# Save job info
echo "$CLASSICAL_JOB" > .job_id
echo "Job ID saved to .job_id"
