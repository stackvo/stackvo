#!/usr/bin/env bash

# Load common library for shared paths and variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/common.sh"

COMMAND=$1
shift

case "$COMMAND" in
    generate)
        SUBCOMMAND=$1
        shift
        case "$SUBCOMMAND" in
            projects)
                bash "$CLI_DIR/commands/generate.sh" projects
                ;;
            services)
                bash "$CLI_DIR/commands/generate.sh" services
                ;;
            "")
                # No subcommand, generate everything
                bash "$CLI_DIR/commands/generate.sh" "$@"
                ;;
            *)
                echo "Unknown generate subcommand: $SUBCOMMAND"
                echo ""
                echo "Usage:"
                echo "  stackvo generate              → generate everything"
                echo "  stackvo generate projects     → generate only projects"
                echo "  stackvo generate services     → generate only services"
                exit 1
                ;;
        esac
        ;;

    up)
        # Only pull images that don't exist locally (avoids rate limits)
        # Use plain progress mode for cleaner build output
        export BUILDKIT_PROGRESS=plain
        echo "🚀 Starting Stackvo services..."
        docker compose "${COMPOSE_FILES[@]}" up -d --pull=missing --quiet-pull 2>&1 | grep -v "^#" | grep -v "^DEPRECATED" || true
        echo "✅ Stackvo services started successfully!"
        ;;

    down)
        docker compose "${COMPOSE_FILES[@]}" down
        ;;

    restart)
        docker compose "${COMPOSE_FILES[@]}" restart
        ;;

    ps)
        docker compose "${COMPOSE_FILES[@]}" ps
        ;;

    logs)
        docker compose "${COMPOSE_FILES[@]}" logs -f "$@"
        ;;

    doctor)
        bash "$CLI_DIR/support/doctor.sh"
        ;;

    install)
        sudo bash "$CLI_DIR/commands/install.sh"
        ;;

    uninstall)
        sudo bash "$CLI_DIR/commands/uninstall.sh"
        ;;

    pull)
        bash "$CLI_DIR/commands/pull.sh"
        ;;

    *)
        echo "Stackvo CLI"
        echo ""
        echo "Available commands:"
        echo "  stackvo install               → install Stackvo CLI"
        echo "  stackvo generate              → generate dynamic compose (all)"
        echo "  stackvo generate projects     → generate only projects"
        echo "  stackvo generate services     → generate only services"
        echo "  stackvo up                    → start all systems"
        echo "  stackvo down                  → stop the system"
        echo "  stackvo restart               → restart services"
        echo "  stackvo ps                    → list running services"
        echo "  stackvo logs [srv]            → follow logs"
        echo "  stackvo pull                  → pull Docker images"
        echo "  stackvo doctor                → system health check"
        echo "  stackvo uninstall             → uninstall Stackvo"
        echo ""
        exit 1
        ;;
esac
