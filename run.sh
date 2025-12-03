#!/bin/bash
# Simple runner script for Ada

if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: OPENAI_API_KEY environment variable is not set."
    echo "Please set it with: export OPENAI_API_KEY=sk-your-key-here"
    exit 1
fi

cargo run --release
