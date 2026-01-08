#!/bin/bash
set -e

echo "Running database migrations..."

# Wait a moment to ensure the database is fully ready
sleep 2

# Run all migration files in order
for migration in /app/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "Running migration: $(basename $migration)"
        psql "$DATABASE_URL" -f "$migration"
        if [ $? -eq 0 ]; then
            echo "✓ Migration $(basename $migration) completed successfully"
        else
            echo "✗ Migration $(basename $migration) failed"
            exit 1
        fi
    fi
done

echo "All migrations completed successfully!"
echo "Starting application..."

# Execute the main application
exec "$@"
