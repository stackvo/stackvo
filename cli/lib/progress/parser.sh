#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/state.sh"

##
# Docker Compose çıktısını parse eder - OPTIMIZE EDİLMİŞ
##

# Global değişkenler
LAST_IMAGE=""
LAST_BUILD=""

##
# Byte değerini parse eder - OPTIMIZE EDİLMİŞ
# Parametreler:
#   $1 - Byte string (örn: "2.1MB", "512KB")
# Çıktı:
#   Byte sayısı
##
parse_bytes() {
    local str=$1
    
    # Lookup table ile hızlı dönüşüm
    case "${str: -2}" in
        MB)
            local num="${str%MB}"
            echo "$((${num%.*} * 1048576))"  # 1024*1024
            ;;
        KB)
            local num="${str%KB}"
            echo "$((${num%.*} * 1024))"
            ;;
        GB)
            local num="${str%GB}"
            echo "$((${num%.*} * 1073741824))"  # 1024*1024*1024
            ;;
        *)
            echo "0"
            ;;
    esac
}

##
# Docker Compose satırını parse eder - OPTIMIZE EDİLMİŞ
# Parametreler:
#   $1 - Docker Compose çıktı satırı
##
parse_docker_line() {
    local line=$1
    
    # OPTİMİZASYON: Önce basit string kontrolü (regex'ten önce)
    
    # Image Pulling (en sık görülen)
    # Sadece gerçek image adlarını yakala (layer ID'leri değil)
    if [[ "$line" == *" Pulling"* ]]; then
        if [[ "$line" =~ ([a-zA-Z][a-zA-Z0-9_-]+)\ Pulling ]]; then
            local image="${BASH_REMATCH[1]}"
            # Layer ID'leri filtrele (sadece hex karakterler)
            if [[ ! "$image" =~ ^[0-9a-f]{12}$ ]]; then
                update_image_status "$image" "downloading" 0 0
                LAST_IMAGE="$image"
            fi
            return
        fi
    fi
    
    # Downloading progress (en sık görülen #2)
    if [[ "$line" == *"Downloading"* ]]; then
        # Basitleştirilmiş regex
        if [[ "$line" =~ ([0-9.]+[KMGT]?B)/([0-9.]+[KMGT]?B) ]]; then
            local current=$(parse_bytes "${BASH_REMATCH[1]}")
            local total=$(parse_bytes "${BASH_REMATCH[2]}")
            [ -n "$LAST_IMAGE" ] && update_image_status "$LAST_IMAGE" "downloading" "$current" "$total"
            return
        fi
    fi
    
    # Download complete
    if [[ "$line" == *"Download complete"* ]]; then
        [ -n "$LAST_IMAGE" ] && update_image_status "$LAST_IMAGE" "extracting" 0 0
        return
    fi
    
    # Extracting
    if [[ "$line" == *"Extracting"* ]]; then
        [ -n "$LAST_IMAGE" ] && update_image_status "$LAST_IMAGE" "extracting" 50 100
        return
    fi
    
    # Pull complete
    if [[ "$line" == *"Pull complete"* ]]; then
        [ -n "$LAST_IMAGE" ] && update_image_status "$LAST_IMAGE" "complete" 100 100
        return
    fi
    
    # Pulled (tüm image tamamlandı)
    if [[ "$line" == *" Pulled"* ]]; then
        if [[ "$line" =~ ([a-zA-Z][a-zA-Z0-9_-]+)\ Pulled ]]; then
            local image="${BASH_REMATCH[1]}"
            # Layer ID'leri filtrele
            if [[ ! "$image" =~ ^[0-9a-f]{12}$ ]]; then
                update_image_status "$image" "complete" 100 100
            fi
            return
        fi
    fi
    
    # Building
    if [[ "$line" == *" Building"* ]]; then
        if [[ "$line" =~ ([a-zA-Z0-9_-]+)\ Building ]]; then
            local service="${BASH_REMATCH[1]}"
            update_build_status "$service" 0 5 "Başlatılıyor..."
            LAST_BUILD="$service"
            return
        fi
    fi
    
    # Build steps (BuildKit format) - Sadece # ile başlayanlar
    if [[ "${line:0:1}" == "#" ]]; then
        # Basitleştirilmiş regex
        if [[ "$line" =~ ^\#([0-9]+) ]]; then
            local step_num="${BASH_REMATCH[1]}"
            [ -n "$LAST_BUILD" ] && update_build_status "$LAST_BUILD" "$step_num" 5 "İşleniyor..."
            return
        fi
    fi
    
    # Built
    if [[ "$line" == *" Built"* ]]; then
        if [[ "$line" =~ ([a-zA-Z0-9_-]+)\ Built ]]; then
            complete_build "${BASH_REMATCH[1]}"
            return
        fi
    fi
}
