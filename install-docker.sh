#!/bin/bash

echo "🐋 Installing Docker..."
echo ""

# Remove old versions if any
echo "📦 Removing old Docker versions..."
sudo apt-get remove docker docker-engine docker.io containerd runc 2>/dev/null || true

# Install prerequisites
echo "📦 Installing prerequisites..."
sudo apt-get update
sudo apt-get install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker's official GPG key
echo "🔑 Adding Docker GPG key..."
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Set up the repository
echo "📝 Adding Docker repository..."
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine
echo "🐋 Installing Docker Engine..."
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Add current user to docker group
echo "👤 Adding user to docker group..."
sudo usermod -aG docker $USER

# Start Docker
echo "🚀 Starting Docker..."
sudo systemctl start docker
sudo systemctl enable docker

# Verify installation
echo "✅ Verifying Docker installation..."
sudo docker --version
sudo docker compose version

echo ""
echo "🎉 Docker installed successfully!"
echo ""
echo "⚠️  IMPORTANT: You need to log out and log back in for group changes to take effect"
echo "   Or run: newgrp docker"
echo ""
