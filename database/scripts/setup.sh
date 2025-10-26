#!/bin/bash
# Database setup script

set -e

echo "🚀 Setting up MIDI Library Database..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Stop existing containers
echo "🛑 Stopping existing containers..."
docker-compose down

# Start containers
echo "🐳 Starting PostgreSQL and Meilisearch..."
docker-compose up -d

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker exec midi-library-postgres pg_isready -U midiuser -d midi_library > /dev/null 2>&1; do
    sleep 1
done

echo "✅ PostgreSQL is ready!"

# Run migrations
echo "📝 Running database migrations..."
for migration in migrations/*.sql; do
    echo "  Running $(basename $migration)..."
    docker exec -i midi-library-postgres psql -U midiuser -d midi_library < "$migration"
done

echo "✅ Migrations complete!"

# Test connection
echo "🔍 Testing database connection..."
if docker exec midi-library-postgres psql -U midiuser -d midi_library -c "SELECT 1" > /dev/null 2>&1; then
    echo "✅ Database connection successful!"
else
    echo "❌ Database connection failed!"
    exit 1
fi

# Show database info
echo ""
echo "📊 Database Information:"
docker exec midi-library-postgres psql -U midiuser -d midi_library -c "
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Connection details:"
echo "  Host: localhost"
echo "  Port: 5433"
echo "  Database: midi_library"
echo "  User: midiuser"
echo "  Password: 145278963"
echo ""
echo "Connect with:"
echo "  PGPASSWORD=145278963 psql -h localhost -p 5433 -U midiuser -d midi_library"
