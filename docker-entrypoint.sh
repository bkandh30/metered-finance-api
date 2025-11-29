#!/bin/sh
set -e

echo "Running database migrations..."
sqlx migrate run

echo "Starting application..."
exec "$@"