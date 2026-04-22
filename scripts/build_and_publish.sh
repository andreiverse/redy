#!/bin/bash
set -e

# Configuration
GHCR_BASE="ghcr.io/andreiverse/newsandreivip"
VERSION=${1:-latest}

# 1. Build Server
echo "[$(date +%T)] Building Server Image..."
# Build from root because server depends on ../migration
docker build -t redy-server:latest -f server/Dockerfile .

# 2. Build Client
echo "[$(date +%T)] Building Client Image..."
docker build -t redy-client:latest -f client/Dockerfile client

# 3. Build ML Worker
echo "[$(date +%T)] Building ML Worker Image..."
docker build -t redy-ml-worker:latest -f ml-worker/Dockerfile .

# 4. Tag and Publish Server
echo "[$(date +%T)] Tagging and Publishing Server..."
docker tag redy-server:latest ${GHCR_BASE}-server:${VERSION}
if [ "$VERSION" != "latest" ]; then
    docker tag redy-server:latest ${GHCR_BASE}-server:latest
fi

# 5. Tag and Publish Client
echo "[$(date +%T)] Tagging and Publishing Client..."
docker tag redy-client:latest ${GHCR_BASE}-client:${VERSION}
if [ "$VERSION" != "latest" ]; then
    docker tag redy-client:latest ${GHCR_BASE}-client:latest
fi

# 6. Tag and Publish ML Worker
echo "[$(date +%T)] Tagging and Publishing ML Worker..."
docker tag redy-ml-worker:latest ${GHCR_BASE}-ml-worker:${VERSION}
if [ "$VERSION" != "latest" ]; then
    docker tag redy-ml-worker:latest ${GHCR_BASE}-ml-worker:latest
fi

# Push to GHCR
echo "[$(date +%T)] Pushing images to GHCR..."
docker push ${GHCR_BASE}-server:${VERSION}
docker push ${GHCR_BASE}-client:${VERSION}
docker push ${GHCR_BASE}-ml-worker:${VERSION}
if [ "$VERSION" != "latest" ]; then
    docker push ${GHCR_BASE}-server:latest
    docker push ${GHCR_BASE}-client:latest
    docker push ${GHCR_BASE}-ml-worker:latest
fi

echo "[$(date +%T)] Done! Images published to:"
echo "  ${GHCR_BASE}-server:${VERSION}"
echo "  ${GHCR_BASE}-client:${VERSION}"
echo "  ${GHCR_BASE}-ml-worker:${VERSION}"
