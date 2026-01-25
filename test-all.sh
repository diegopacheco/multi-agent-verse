#!/bin/bash
set -e

echo "Running Bun/Vitest tests..."
cd frontend
bun run test
cd ..

echo ""
echo "Running Rust tests..."
cd backend
cargo test
cd ..

echo ""
echo "Done."
