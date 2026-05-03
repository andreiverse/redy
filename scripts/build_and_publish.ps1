$ErrorActionPreference = "Stop"

# Configuration
$GHCR_BASE = "ghcr.io/andreiverse/newsandreivip"
$VERSION = if ($args[0]) { $args[0] } else { "latest" }

# 1. Build Server
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Building Server Image..."
# Build from root because server depends on ../migration
docker build -t redy-server:latest -f server/Dockerfile .

# 2. Build Client
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Building Client Image..."
docker build -t redy-client:latest -f client/Dockerfile client

# 3. Build ML Worker
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Building ML Worker Image..."
docker build -t redy-ml-worker:latest -f ml-worker/Dockerfile .

# 4. Tag and Publish Server
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Tagging and Publishing Server..."
docker tag redy-server:latest "${GHCR_BASE}-server:${VERSION}"
if ($VERSION -ne "latest") {
    docker tag redy-server:latest "${GHCR_BASE}-server:latest"
}

# 5. Tag and Publish Client
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Tagging and Publishing Client..."
docker tag redy-client:latest "${GHCR_BASE}-client:${VERSION}"
if ($VERSION -ne "latest") {
    docker tag redy-client:latest "${GHCR_BASE}-client:latest"
}

# 6. Tag and Publish ML Worker
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Tagging and Publishing ML Worker..."
docker tag redy-ml-worker:latest "${GHCR_BASE}-ml-worker:${VERSION}"
if ($VERSION -ne "latest") {
    docker tag redy-ml-worker:latest "${GHCR_BASE}-ml-worker:latest"
}

# Push to GHCR
Write-Host "[$(Get-Date -Format "HH:mm:ss")] Pushing images to GHCR..."
docker push "${GHCR_BASE}-server:${VERSION}"
docker push "${GHCR_BASE}-client:${VERSION}"
docker push "${GHCR_BASE}-ml-worker:${VERSION}"
if ($VERSION -ne "latest") {
    docker push "${GHCR_BASE}-server:latest"
    docker push "${GHCR_BASE}-client:latest"
    docker push "${GHCR_BASE}-ml-worker:latest"
}

Write-Host "[$(Get-Date -Format "HH:mm:ss")] Done! Images published to:"
Write-Host "  ${GHCR_BASE}-server:${VERSION}"
Write-Host "  ${GHCR_BASE}-client:${VERSION}"
Write-Host "  ${GHCR_BASE}-ml-worker:${VERSION}"
