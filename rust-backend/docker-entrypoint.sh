#!/bin/sh
set -e

# Dynamically add appuser to the docker socket's group
if [ -S /var/run/docker.sock ]; then
    DOCKER_GID=$(stat -c '%g' /var/run/docker.sock)
    if ! getent group "$DOCKER_GID" > /dev/null 2>&1; then
        addgroup -g "$DOCKER_GID" -S docker
    fi
    DOCKER_GROUP=$(getent group "$DOCKER_GID" | cut -d: -f1)
    adduser appuser "$DOCKER_GROUP" 2>/dev/null || true
fi

# Drop to appuser and run the application
exec su-exec appuser /app/proton-vpn-api "$@"
