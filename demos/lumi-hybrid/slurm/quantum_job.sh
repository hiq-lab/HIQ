#!/bin/bash
#SBATCH --job-name=hiq_vqe_quantum
#SBATCH --account=project_462000xxx
#SBATCH --partition=q_fiqci
#SBATCH --time=00:30:00
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --output=quantum_%j.out
#SBATCH --error=quantum_%j.err

# ╔════════════════════════════════════════════════════════════════════════════╗
# ║  LUMI-Q Quantum Job Script                                                  ║
# ║  Executes quantum circuits on IQM quantum computer via HIQ                  ║
# ╚════════════════════════════════════════════════════════════════════════════╝
#
# This script runs on the LUMI quantum partition (q_fiqci) and executes
# quantum circuits for the VQE algorithm.
#
# Usage:
#   sbatch --export=JOB_FILE=job_001.json,OUTPUT_FILE=result_001.json quantum_job.sh
#
# Environment:
#   - HELMI_TOKEN or IQM_TOKEN must be set for authentication
#   - HIQ_DIR: Path to HIQ installation (default: $HOME/hiq)

echo "╔════════════════════════════════════════════════════════════════════════════╗"
echo "║  LUMI-Q HIQ Quantum Worker                                                  ║"
echo "╠════════════════════════════════════════════════════════════════════════════╣"
echo "║  Job ID: $SLURM_JOB_ID"
echo "║  Node:   $SLURMD_NODENAME"
echo "║  Time:   $(date)"
echo "╚════════════════════════════════════════════════════════════════════════════╝"

# Load modules
module load LUMI/23.09
module load cray-python/3.10.10

# Set paths
HIQ_DIR="${HIQ_DIR:-$HOME/hiq}"
WORK_DIR="${SLURM_SUBMIT_DIR:-$(pwd)}"

# Input/output files
JOB_FILE="${JOB_FILE:-$WORK_DIR/quantum_job.json}"
OUTPUT_FILE="${OUTPUT_FILE:-$WORK_DIR/quantum_result.json}"

# Check for authentication token
if [[ -z "$HELMI_TOKEN" && -z "$IQM_TOKEN" ]]; then
    echo "WARNING: No IQM authentication token found"
    echo "Set HELMI_TOKEN or IQM_TOKEN for real quantum execution"
    echo "Falling back to simulator mode"
fi

echo ""
echo "Configuration:"
echo "  HIQ_DIR:     $HIQ_DIR"
echo "  JOB_FILE:    $JOB_FILE"
echo "  OUTPUT_FILE: $OUTPUT_FILE"
echo ""

# Check job file exists
if [[ ! -f "$JOB_FILE" ]]; then
    echo "ERROR: Job file not found: $JOB_FILE"
    exit 1
fi

# Run quantum worker
echo "Starting quantum worker..."
cd "$WORK_DIR"

"$HIQ_DIR/target/release/quantum_worker" \
    --job "$JOB_FILE" \
    --output "$OUTPUT_FILE" \
    --verbose

EXIT_CODE=$?

echo ""
echo "Quantum worker completed with exit code: $EXIT_CODE"
echo "Results written to: $OUTPUT_FILE"

# Show result summary
if [[ -f "$OUTPUT_FILE" ]]; then
    echo ""
    echo "Result summary:"
    python3 -c "
import json
with open('$OUTPUT_FILE') as f:
    r = json.load(f)
print(f\"  Status: {r['status']}\")
print(f\"  Shots:  {r['shots']}\")
print(f\"  Time:   {r['execution_time_ms']} ms\")
if r.get('counts'):
    print(f\"  Counts: {len(r['counts'])} unique outcomes\")
" 2>/dev/null || echo "  (could not parse result)"
fi

exit $EXIT_CODE
