#!/bin/bash
# Dracolich Self-Evolution Loop
# Waits 1 hour between runs, up to 8 hours total
# Usage: ./evolve.sh
# Stop: Ctrl+C or kill the process

set -e

DRACO_DIR="/mnt/e/Dev/Draco"
PROMPT_FILE="prompts/self-improve.md"
MAX_HOURS=8
WAIT_SECONDS=3600  # 1 hour

cd "$DRACO_DIR"

START_TIME=$(date +%s)
END_TIME=$((START_TIME + MAX_HOURS * 3600))
ITERATION=0

echo "============================================"
echo "DRACOLICH EVOLUTION LOOP"
echo "============================================"
echo "Started: $(date)"
echo "Will run until: $(date -d @$END_TIME 2>/dev/null || date -r $END_TIME 2>/dev/null || echo "$MAX_HOURS hours from now")"
echo "Wait between runs: $((WAIT_SECONDS / 60)) minutes"
echo "============================================"
echo ""

find_latest_version() {
    ls -d src/v[0-9]* 2>/dev/null | sed 's/src\/v//' | sort -n | tail -1
}

# Initial wait — current run is still going
echo "Waiting $((WAIT_SECONDS / 60)) minutes for current run to finish..."
echo "First iteration at: $(date -d "+${WAIT_SECONDS} seconds" 2>/dev/null || echo "in 1 hour")"
sleep "$WAIT_SECONDS"

while true; do
    NOW=$(date +%s)
    if [ "$NOW" -ge "$END_TIME" ]; then
        echo ""
        echo "============================================"
        echo "TIME LIMIT REACHED ($MAX_HOURS hours)"
        echo "Completed $ITERATION iteration(s)"
        echo "============================================"
        exit 0
    fi

    ITERATION=$((ITERATION + 1))
    LATEST=$(find_latest_version)
    NEXT=$((LATEST + 1))
    REMAINING=$(( (END_TIME - NOW) / 60 ))

    echo ""
    echo "============================================"
    echo "ITERATION $ITERATION — v$LATEST → v$NEXT"
    echo "Time: $(date)"
    echo "Remaining: ${REMAINING}m"
    echo "============================================"
    echo ""

    # Run self-improvement through latest version
    echo "Running: npx tsx src/v${LATEST}/index.ts --file $PROMPT_FILE"
    echo ""

    if npx tsx "src/v${LATEST}/index.ts" --file "$PROMPT_FILE" 2>&1 | tee "output/evolve-v${LATEST}-to-v${NEXT}.log"; then
        echo ""
        echo "✓ Iteration $ITERATION complete (v$LATEST → v$NEXT)"

        # Verify the new version exists
        if [ -d "src/v$NEXT" ]; then
            LINES=$(find "src/v$NEXT" -name "*.ts" -exec cat {} + | wc -l)
            FILES=$(find "src/v$NEXT" -name "*.ts" | wc -l)
            echo "  v$NEXT: $FILES files, $LINES lines"
        else
            echo "  ⚠ v$NEXT directory not created — run may have failed silently"
        fi
    else
        echo ""
        echo "✗ Iteration $ITERATION failed (exit code $?)"
        echo "  Log: output/evolve-v${LATEST}-to-v${NEXT}.log"
    fi

    # Check if we have time for another run
    NOW=$(date +%s)
    if [ "$NOW" -ge "$END_TIME" ]; then
        echo ""
        echo "============================================"
        echo "TIME LIMIT REACHED after $ITERATION iteration(s)"
        echo "Latest version: v$(find_latest_version)"
        echo "============================================"
        exit 0
    fi

    # Wait
    WAIT_UNTIL=$((NOW + WAIT_SECONDS))
    if [ "$WAIT_UNTIL" -gt "$END_TIME" ]; then
        ACTUAL_WAIT=$((END_TIME - NOW))
        echo ""
        echo "Waiting ${ACTUAL_WAIT}s (truncated — time limit approaching)..."
        sleep "$ACTUAL_WAIT"
    else
        echo ""
        echo "Waiting $((WAIT_SECONDS / 60)) minutes until next iteration..."
        echo "Next run: $(date -d "+${WAIT_SECONDS} seconds" 2>/dev/null || echo "in 1 hour")"
        sleep "$WAIT_SECONDS"
    fi
done
