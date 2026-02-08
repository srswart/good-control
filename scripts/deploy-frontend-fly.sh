#!/usr/bin/env bash
# Deploy the inventory web frontend to Fly.io.

set -euo pipefail

APP_NAME="${APP_NAME:-good-control}"
CONFIG_PATH="${CONFIG_PATH:-apps/web/fly.toml}"
REGION="${REGION:-sin}"
ORG="${ORG:-}"
CREATE_APP="false"
RUN_SMOKE_CHECK="true"
CHECK_CONTEXT="true"
CONTEXT_REPORT="false"

usage() {
    echo "Usage: $0 [--create-app] [--region <region>] [--skip-smoke] [--skip-context-check] [--context-report]"
    echo ""
    echo "Environment variables:"
    echo "  APP_NAME     Fly app name (default: good-control)"
    echo "  CONFIG_PATH  Fly config path (default: apps/web/fly.toml)"
    echo "  REGION       Fly region (default: sin)"
    echo "  ORG          Fly org slug (default: current org)"
    echo ""
    echo "Options:"
    echo "  --create-app     Create app if missing"
    echo "  --region <r>     Override region"
    echo "  --skip-smoke     Skip /health smoke check"
    echo "  --skip-context-check Skip context size check"
    echo "  --context-report Print largest files in context"
    echo "  -h, --help       Show this help message"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --create-app)
            CREATE_APP="true"
            shift
            ;;
        --region)
            REGION="${2:-}"
            if [[ -z "$REGION" ]]; then
                echo "Missing value for --region"
                exit 1
            fi
            shift 2
            ;;
        --skip-smoke)
            RUN_SMOKE_CHECK="false"
            shift
            ;;
        --skip-context-check)
            CHECK_CONTEXT="false"
            shift
            ;;
        --context-report)
            CONTEXT_REPORT="true"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if ! command -v fly >/dev/null 2>&1; then
    echo "fly CLI not found. Install with: brew install flyctl"
    exit 1
fi

if [[ ! -f "$CONFIG_PATH" ]]; then
    echo "Fly config not found at $CONFIG_PATH"
    exit 1
fi

echo "Deploying $APP_NAME using $CONFIG_PATH"

if [[ "$CHECK_CONTEXT" == "true" ]]; then
    echo "Estimating Docker build context size (excluding .dockerignore)..."
    if command -v docker >/dev/null 2>&1; then
        if command -v timeout >/dev/null 2>&1; then
            context_size=$(timeout 5s docker buildx du --local . 2>/dev/null | awk 'NR==2 {print $4, $5}')
            if [[ -n "$context_size" ]]; then
                echo "Docker context size: $context_size"
            else
                echo "Docker context size: unavailable (timed out or no data)"
            fi
        else
            echo "timeout command not found; skipping context size estimate."
        fi
    else
        echo "Docker CLI not found; skipping context size estimate."
    fi

    if tar -cf - --exclude-from=.dockerignore . 2>/dev/null | tar -tf - | grep -q '^target/'; then
        echo "Warning: target/ appears in the build context."
    else
        echo "target/ is excluded from the build context."
    fi

    if [[ "$CONTEXT_REPORT" == "true" ]]; then
        if command -v python3 >/dev/null 2>&1; then
            python3 scripts/docker-context-report.py --top 30 --min-mb 1
        else
            echo "python3 not found; skipping context report."
        fi
    fi
fi

if [[ "$CREATE_APP" == "true" ]]; then
    echo "Ensuring app $APP_NAME exists..."
    set +e
    if [[ -n "$ORG" ]]; then
        fly apps create "$APP_NAME" --org "$ORG"
    else
        fly apps create "$APP_NAME"
    fi
    create_status=$?
    set -e
    if [[ $create_status -ne 0 ]]; then
        echo "App creation failed (may already exist). Continuing..."
    fi
fi

fly deploy --config "$CONFIG_PATH" --app "$APP_NAME"

if [[ "$RUN_SMOKE_CHECK" == "true" ]]; then
    echo "Running smoke check..."
    curl -f "https://${APP_NAME}.fly.dev/health"
    echo ""
    echo "Smoke check passed."
fi
