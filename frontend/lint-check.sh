#!/bin/bash

# Frontend Linting Check Script
# Ensures code quality before deployment

echo "🔍 Running ESLint checks..."

# Run linting
if pnpm run lint; then
    echo "✅ All linting checks passed!"
    exit 0
else
    echo "❌ Linting errors found. Please fix before deploying."
    echo ""
    echo "Run 'pnpm run lint' to see detailed errors."
    echo "The build has been configured to automatically run linting."
    exit 1
fi
