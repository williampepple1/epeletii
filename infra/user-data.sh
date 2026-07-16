#!/bin/bash
set -euo pipefail

# Install Docker if missing
if ! command -v docker &>/dev/null; then
  apt-get update -qq
  apt-get install -y -qq ca-certificates curl
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
  chmod a+r /etc/apt/keyrings/docker.asc
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io
fi

# Pull and run the game server
docker pull ${docker_image}

docker rm -f epeletii 2>/dev/null || true

docker run -d \
  --name epeletii \
  --restart unless-stopped \
  -p 9001:9001 \
  -e RUST_LOG=info \
  -e MONGO_URI="${mongo_uri}" \
  -e JWT_SECRET="${jwt_secret}" \
  ${docker_image}

# Clean up old images
docker image prune -f
