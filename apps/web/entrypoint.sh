#!/usr/bin/env sh
set -e

GRAPHQL_ENDPOINT="${APP__GRAPHQL_ENDPOINT:-http://localhost:3000/graphql}"

# Write runtime config to a fixed path (not hashed by asset pipeline)
cat > /usr/share/nginx/html/config.js <<EOF
window.__APP_GRAPHQL_ENDPOINT = "${GRAPHQL_ENDPOINT}";
window.__APP_CONFIG__ = {
    graphqlEndpoint: "${GRAPHQL_ENDPOINT}",
};
EOF

# Inject config.js script tag into index.html before </head>
sed -i 's|</head>|<script src="/config.js"></script></head>|' /usr/share/nginx/html/index.html

exec nginx -g "daemon off;"
