#!/usr/bin/env bash
# VecBase — install_docker.sh
# Installs Docker and docker-compose if not present.

set -euo pipefail

if command -v docker &>/dev/null; then
    echo "Docker already installed: $(docker --version)"
    exit 0
fi

echo "Installing Docker..."
curl -fsSL https://get.docker.com | sh
echo "Docker installed."
