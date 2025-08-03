#!/bin/bash

echo "Starting local server for Hesha Protocol website..."
echo "Website will be available at: http://localhost:8000"
echo "Press Ctrl+C to stop the server"
echo ""

cd "$(dirname "$0")"
python3 -m http.server 8000