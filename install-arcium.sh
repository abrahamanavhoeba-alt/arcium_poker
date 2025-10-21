#!/bin/bash

echo "🚀 Installing Arcium MPC Tooling..."
echo ""

# Step 1: Install system dependencies
echo "📦 Step 1: Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev

# Step 2: Install Arcium
echo "🔧 Step 2: Installing Arcium CLI..."
curl --proto '=https' --tlsv1.2 -sSfL https://install.arcium.com/ | bash

# Step 3: Source the environment
echo "⚙️  Step 3: Setting up environment..."
source "$HOME/.bashrc"
source "$HOME/.cargo/env"

# Step 4: Verify installation
echo "✅ Step 4: Verifying installation..."
if command -v arcium &> /dev/null; then
    echo "✅ Arcium installed successfully!"
    arcium --version
else
    echo "⚠️  Arcium command not found. You may need to:"
    echo "   source ~/.bashrc"
    echo "   or restart your terminal"
fi

echo ""
echo "🎉 Installation complete!"
echo ""
echo "Next steps:"
echo "1. Run: source ~/.bashrc"
echo "2. Verify: arcium --version"
echo "3. Build MPC: cd /home/a/arcium_poker && arcium build"
