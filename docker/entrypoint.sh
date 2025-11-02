#!/bin/bash
set -e

CONFIG_FILE="${CONFIG_FILE:-/config/conf.json}"

# Check if config file exists
if [ ! -f "${CONFIG_FILE}" ]; then
    echo "ERROR: Config file not found at ${CONFIG_FILE}"
    echo ""
    echo "To generate a config file, run:"
    echo "  docker run --rm org-roamers:latest /app/org-roamers-cli get-config > config.json"
    echo ""
    echo "For authentication support:"
    echo "  docker run --rm org-roamers:latest /app/org-roamers-cli get-config --with-auth > config.json"
    echo ""
    echo "Then edit config.json to:"
    echo "  - Set org_roamers_root to /data"
    echo "  - Set host to 0.0.0.0"
    echo "  - Add users if using authentication"
    echo ""
    echo "Finally, mount it when starting the container:"
    echo "  -v ./config.json:/config/conf.json:ro"
    echo ""
    exit 1
fi

echo "Starting org-roamers server..."
echo "  Config: ${CONFIG_FILE}"
echo "  Org files: /data"

exec ./org-roamers-cli server --config "${CONFIG_FILE}"
