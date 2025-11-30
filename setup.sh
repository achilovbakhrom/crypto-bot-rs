#!/bin/bash
set -e

echo "🚀 Setting up Crypto Bot..."

# Check if PostgreSQL is running
if ! docker ps | grep -q postgres; then
    echo "📦 Starting PostgreSQL container..."
    docker run -d \
        --name crypto-bot-postgres \
        -e POSTGRES_PASSWORD=password \
        -e POSTGRES_DB=crypto_bot \
        -p 5432:5432 \
        postgres:14
    
    echo "⏳ Waiting for PostgreSQL to be ready..."
    sleep 5
fi

# Generate encryption key if not exists
if ! grep -q "^ENCRYPTION_KEY=[0-9a-f]\{64\}$" .env 2>/dev/null; then
    echo "🔐 Generating encryption key..."
    KEY=$(openssl rand -hex 32)
    sed -i.bak "s/^ENCRYPTION_KEY=.*/ENCRYPTION_KEY=$KEY/" .env
    echo "✅ Encryption key generated and saved to .env"
fi

echo ""
echo "✨ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Review and update .env file if needed"
echo "2. Build: cargo build --release"
echo "3. Run: cargo run --release"
echo ""
echo "API will be available at: http://localhost:8080"
