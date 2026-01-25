#!/bin/bash
cd "$(dirname "$0")"
cd backend && cargo build --release
cd ../frontend && npm install && npm run build
