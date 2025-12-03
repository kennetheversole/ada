#!/bin/bash
# Test config creation
cargo run --release --quiet 2>&1 | head -5 &
PID=$!
sleep 2
kill $PID 2>/dev/null
echo ""
echo "Checking for config file..."
if [ -f ~/.ada/config ]; then
    echo "Config file created successfully!"
    echo "Contents:"
    cat ~/.ada/config
else
    echo "Config file not found"
fi
