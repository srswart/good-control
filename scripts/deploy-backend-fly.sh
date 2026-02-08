#!/usr/bin/env bash
# Deploy the inventory backend to Fly.io.

set -euo pipefail

APP_NAME="${APP_NAME:-good-control-api}"
CONFIG_PATH="${CONFIG_PATH:-apps/server/fly.toml}"
VOLUME_NAME="${VOLUME_NAME:-inventory_data}"
REGION="${REGION:-iad}"
ORG="${ORG:-}"
CREATE_APP="false"
CREATE_VOLUME="false"
RUN_SMOKE_CHECK="true"
CHECK_CONTEXT="true"
CONTEXT_REPORT="false"

usage() {
    echo "Usage: $0 [--create-app] [--create-volume] [--region <region>] [--skip-smoke] [--skip-context-check] [--context-report]"
    echo ""
    echo "Environment variables:"
    echo "  APP_NAME     Fly app name (default: good-control-api)"
    echo "  CONFIG_PATH  Fly config path (default: apps/server/fly.toml)"
    echo "  VOLUME_NAME  Fly volume name (default: inventory_data)"
    echo "  REGION       Fly region for volume (default: iad)"
    echo "  ORG          Fly org slug (default: current org)"
    echo ""
    echo "Options:"
    echo "  --create-app     Create app if missing"
    echo "  --create-volume  Create volume before deploy"
    echo "  --region <r>     Override volume region"
    echo "  --skip-smoke     Skip /health smoke check"
    echo "  --skip-context-check Skip context size check"
    echo "  --context-report Print largest files in context"
    echo "  -h, --help       Show this help message"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --create-volume)
            CREATE_VOLUME="true"
            shift
            ;;
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

if [[ "$CREATE_VOLUME" == "true" ]]; then
    echo "Ensuring volume $VOLUME_NAME exists in $REGION..."
    existing_volume=$(fly volumes list --app "$APP_NAME" 2>/dev/null | awk -v name="$VOLUME_NAME" -v region="$REGION" 'NR>1 && $3==name && $5==region {print $1; exit}')
    if [[ -n "$existing_volume" ]]; then
        echo "Found existing volume: $existing_volume (region $REGION)"
    else
        set +e
        fly volumes create "$VOLUME_NAME" --app "$APP_NAME" --size 1 --region "$REGION"
        create_status=$?
        set -e
        if [[ $create_status -ne 0 ]]; then
            echo "Volume creation failed. Check volume limits and region capacity."
        fi
    fi
fi

fly deploy --config "$CONFIG_PATH" --app "$APP_NAME"

if [[ "$RUN_SMOKE_CHECK" == "true" ]]; then
    echo "Running smoke check..."
    curl -f "https://${APP_NAME}.fly.dev/health"
    echo ""
    echo "Smoke check passed."
fi
