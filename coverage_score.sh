#!/bin/bash

# Run tarpaulin and capture output
output=$(cargo tarpaulin --ignore-tests --line --quiet 2>/dev/null)

# Extract coverage percentage (assumes output like "Coverage Results: 84.52%")
percentage=$(echo "$output" | grep -oP 'Coverage Results: \K[0-9.]+')

if [[ -z "$percentage" ]]; then
    echo "Tarpaulin did not return coverage. Score: 0"
    exit 1
fi

# Score based on thresholds
if (( $(echo "$percentage > 80.0" | bc -l) )); then
    echo "Coverage: $percentage% — Score: 3"
elif (( $(echo "$percentage > 60.0" | bc -l) )); then
    echo "Coverage: $percentage% — Score: 2"
elif (( $(echo "$percentage > 40.0" | bc -l) )); then
    echo "Coverage: $percentage% — Score: 1"
else
    echo "Coverage: $percentage% — Score: 0"
fi

