#!/bin/bash
cd "$(dirname "$0")"
cd backend && cargo build --release
cd ../frontend && bun install && bun run build
