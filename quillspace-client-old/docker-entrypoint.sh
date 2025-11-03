#!/bin/sh
set -e

# Replace environment variables in built files if needed
if [ -n "$VITE_API_BASE_URL" ]; then
    echo "Setting API base URL to: $VITE_API_BASE_URL"
    # You can add runtime environment variable replacement here if needed
fi

# Start nginx
exec "$@"
