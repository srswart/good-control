#!/usr/bin/env bash
# Load sample inventory data into the database

set -e

# Default database path
DB_PATH="${1:-data/drone-inventory.db}"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "Error: Database not found at $DB_PATH"
    echo "Please run the server first to create the database and run migrations:"
    echo "  ./scripts/run-server.sh"
    exit 1
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEED_FILE="$SCRIPT_DIR/sample_inventory.sql"

# Check if seed file exists
if [ ! -f "$SEED_FILE" ]; then
    echo "Error: Seed file not found at $SEED_FILE"
    exit 1
fi

echo "Loading sample inventory data..."
echo "Database: $DB_PATH"
echo "Seed file: $SEED_FILE"
echo ""

# Load the seed data using sqlite3
if command -v sqlite3 &> /dev/null; then
    sqlite3 "$DB_PATH" < "$SEED_FILE"
    echo "✅ Sample data loaded successfully!"
    echo ""
    echo "Sample data includes:"
    echo "  - 10 items (drones, batteries, parts, tools, packaging, consumables)"
    echo "  - 10 locations across warehouses WH-SZ and WH-BJ"
    echo "  - 10 inventory movements (receipts)"
    echo "  - 10 inventory positions (current stock levels)"
    echo "  - 10 serial units (for serialized items)"
    echo "  - 2 lots (for lot-tracked items)"
    echo ""
    echo "You can now query items via GraphQL!"
else
    echo "Error: sqlite3 command not found"
    echo "Please install sqlite3 to load seed data:"
    echo "  macOS: brew install sqlite"
    echo "  Ubuntu/Debian: sudo apt-get install sqlite3"
    exit 1
fi

