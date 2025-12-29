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
        # Parse flags for selective startup
        START_MODE="minimal"  # Default: minimal (core only)
        CUSTOM_PROFILES=()
        
        while [[ $# -gt 0 ]]; do
            case "$1" in
                --all)
                    START_MODE="all"
                    shift
                    ;;
                --services)
                    START_MODE="services"
                    shift
                    ;;
                --projects)
                    START_MODE="projects"
                    shift
                    ;;
                --profile)
                    CUSTOM_PROFILES+=("$2")
                    shift 2
                    ;;
                *)
                    shift
                    ;;
            esac
        done
        
        # Build profile arguments
        PROFILE_ARGS=""
        case "$START_MODE" in
            minimal)
                PROFILE_ARGS="--profile core"
                echo "🚀 Starting Stackvo (minimal mode: core services only)..."
                ;;
            services)
                PROFILE_ARGS="--profile core --profile services"
                echo "🚀 Starting Stackvo (core + services)..."
                ;;
            projects)
                PROFILE_ARGS="--profile core --profile projects"
                echo "🚀 Starting Stackvo (core + projects)..."
                ;;
            all)
                PROFILE_ARGS="--profile core --profile services --profile projects"
                echo "🚀 Starting Stackvo (all services and projects)..."
                ;;
        esac
        
        # Add custom profiles if specified
        for profile in "${CUSTOM_PROFILES[@]}"; do
            PROFILE_ARGS="$PROFILE_ARGS --profile $profile"
            echo "  + Including profile: $profile"
        done
        
        # Start with profiles
        export BUILDKIT_PROGRESS=plain
        docker compose "${COMPOSE_FILES[@]}" $PROFILE_ARGS up -d --build --pull=missing --remove-orphans 2>&1 | grep -v "^#" | grep -v "^DEPRECATED" || true
        echo "✅ Stackvo started successfully!"
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
        echo "  stackvo up                    → start core services (minimal)"
        echo "  stackvo up --services         → start core + all services"
        echo "  stackvo up --projects         → start core + all projects"
        echo "  stackvo up --all              → start everything (old behavior)"
        echo "  stackvo up --profile <name>   → start core + specific profile"
        echo "  stackvo down                  → stop the system"
        echo "  stackvo restart               → restart services"
        echo "  stackvo ps                    → list running services"
        echo "  stackvo logs [srv]            → follow logs"
        echo "  stackvo pull                  → pull Docker images"
        echo "  stackvo doctor                → system health check"
        echo "  stackvo uninstall             → uninstall Stackvo"
        echo ""
        echo "Examples:"
        echo "  stackvo up                    → Start only Traefik + UI"
        echo "  stackvo up --profile mysql    → Start core + MySQL only"
        echo "  stackvo up --profile project-myproject  → Start core + myproject only"
        echo ""
        exit 1
        ;;
esac
