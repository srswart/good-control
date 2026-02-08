#!/usr/bin/env bash
# Run the inventory GraphQL server with configurable log level

set -e

# Default log level
LOG_LEVEL="info"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            LOG_LEVEL="debug"
            shift
            ;;
        --info)
            LOG_LEVEL="info"
            shift
            ;;
        --trace)
            LOG_LEVEL="trace"
            shift
            ;;
        --warn)
            LOG_LEVEL="warn"
            shift
            ;;
        --error)
            LOG_LEVEL="error"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--debug|--info|--trace|--warn|--error]"
            echo ""
            echo "Options:"
            echo "  --debug    Set log level to debug (verbose)"
            echo "  --info     Set log level to info (default)"
            echo "  --trace    Set log level to trace (very verbose)"
            echo "  --warn     Set log level to warn"
            echo "  --error    Set log level to error"
            echo "  -h, --help Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  APP__LOG_LEVEL    Override log level (e.g., APP__LOG_LEVEL=debug)"
            echo "  APP__SERVER__HOST Override server host (default: 127.0.0.1)"
            echo "  APP__SERVER__PORT Override server port (default: 3000)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment variable for log level
export APP__LOG_LEVEL="$LOG_LEVEL"

echo "Starting inventory GraphQL server..."
echo "Log level: $LOG_LEVEL"
echo ""

# Build and run the server
cargo run --bin inventory-server

