#!/bin/sh

# Replace environment variables in built files
if [ -n "$VITE_API_BASE_URL" ]; then
    find /usr/share/nginx/html -name "*.js" -exec sed -i "s|http://localhost:3000/api/v1|$VITE_API_BASE_URL|g" {} \;
fi

# Start nginx
exec "$@"
