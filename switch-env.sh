#!/bin/bash
# Switch between development and production environments

set -e

ENVIRONMENT=$1

if [ -z "$ENVIRONMENT" ]; then
    echo "Usage: ./switch-env.sh [dev|prod]"
    echo ""
    echo "Current environment:"
    if [ -f .env ]; then
        grep "^APP_ENV=" .env || echo "Unknown"
    else
        echo ".env file not found"
    fi
    exit 1
fi

case $ENVIRONMENT in
    dev|development)
        echo "Switching to DEVELOPMENT environment..."
        cp .env.development .env
        echo "Switched to development"
        echo ""
        echo "Database: Development branch on Neon"
        echo "Logs: Debug level"
        echo "Rate limits: Relaxed"
        ;;
    prod|production)
        echo "Switching to PRODUCTION environment..."
        cp .env.production .env
        echo "Switched to production"
        echo ""
        echo "WARNING: You are now in PRODUCTION mode!"
        echo "Database: Production on Neon"
        echo "Logs: Warn level"
        echo "Rate limits: Strict"
        ;;
    *)
        echo "Invalid environment: $ENVIRONMENT"
        echo "Use: dev, development, prod, or production"
        exit 1
        ;;
esac

echo ""
echo "Next steps:"
echo "  docker-compose restart api"