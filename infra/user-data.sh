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

# Create a docker network for containers to communicate
docker network create epeletii-net || true

# Start MongoDB container
docker pull mongo:7.0
docker rm -f epeletii-mongo 2>/dev/null || true
docker run -d \
  --name epeletii-mongo \
  --network epeletii-net \
  --restart unless-stopped \
  -v epeletii_mongo_data:/data/db \
  mongo:7.0

# Pull and run the game server
docker pull ${docker_image}
docker rm -f epeletii 2>/dev/null || true
docker run -d \
  --name epeletii \
  --network epeletii-net \
  --restart unless-stopped \
  -p 9001:9001 \
  -e RUST_LOG=info \
  -e MONGO_URI="mongodb://epeletii-mongo:27017/epeletii" \
  -e JWT_SECRET="${jwt_secret}" \
  ${docker_image}

# Clean up old images
docker image prune -f

# Install and configure Nginx for reverse proxying WebSocket traffic on port 80
apt-get install -y nginx

cat > /etc/nginx/sites-available/epeletii << 'EOF'
server {
    listen 80;
    server_name api.ibani.online;

    location / {
        proxy_pass http://127.0.0.1:9001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Increase timeouts for long-lived WebSocket connections
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
EOF

ln -sf /etc/nginx/sites-available/epeletii /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
systemctl restart nginx
