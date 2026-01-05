#!/usr/bin/env bash

##
# Terminal renderer - Renkli çıktı ve progress bar
# Optimize edilmiş versiyon
##

# ANSI Renk Kodları
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[0;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_MAGENTA='\033[0;35m'
readonly COLOR_CYAN='\033[0;36m'
readonly COLOR_WHITE='\033[0;37m'
readonly COLOR_BOLD='\033[1m'
readonly COLOR_DIM='\033[2m'

# Unicode Karakterler
readonly CHAR_CHECKMARK='✅'
readonly CHAR_DOWNLOAD='📥'
readonly CHAR_BUILD='🔨'
readonly CHAR_PACKAGE='📦'
readonly CHAR_ROCKET='🚀'
readonly CHAR_GEAR='⚙️'
readonly CHAR_CLOCK='⏱️'
readonly CHAR_ERROR='❌'
readonly CHAR_WARNING='⚠️'

##
# Terminal'i temizler ve cursor'ı başa alır
##
clear_screen() {
    printf '\033[2J\033[H'
}

##
# Progress bar oluşturur
# Parametreler:
#   $1 - Mevcut değer
#   $2 - Maksimum değer
#   $3 - Bar genişliği (karakter sayısı)
# Çıktı:
#   Progress bar string (örn: "████████░░░░░░░░░░░░")
##
create_progress_bar() {
    local current=$1
    local max=$2
    local width=${3:-20}
    
    # Yüzde hesapla
    local percent=0
    if [ "$max" -gt 0 ]; then
        percent=$((current * 100 / max))
    fi
    
    # Dolu kısım
    local filled=$((percent * width / 100))
    local empty=$((width - filled))
    
    # Bar oluştur
    local bar=""
    for ((i=0; i<filled; i++)); do
        bar+="█"
    done
    for ((i=0; i<empty; i++)); do
        bar+="░"
    done
    
    echo "$bar"
}

##
# Byte'ı okunabilir formata çevirir
# Parametreler:
#   $1 - Byte sayısı
# Çıktı:
#   Formatlanmış string (örn: "2.5MB")
##
format_bytes() {
    local bytes=$1
    
    if [ "$bytes" -lt 1024 ]; then
        echo "${bytes}B"
    elif [ "$bytes" -lt $((1024 * 1024)) ]; then
        echo "$((bytes / 1024))KB"
    elif [ "$bytes" -lt $((1024 * 1024 * 1024)) ]; then
        local mb=$((bytes / 1024 / 1024))
        echo "${mb}MB"
    else
        local gb=$((bytes / 1024 / 1024 / 1024))
        echo "${gb}GB"
    fi
}

##
# Süreyi formatlar (saniye -> MM:SS)
# Parametreler:
#   $1 - Saniye
# Çıktı:
#   Formatlanmış süre (örn: "01:23")
##
format_duration() {
    local seconds=$1
    local minutes=$((seconds / 60))
    local secs=$((seconds % 60))
    printf "%02d:%02d" "$minutes" "$secs"
}

##
# Başlık çizer
# Parametreler:
#   $1 - Başlık metni
##
render_header() {
    local title=$1
    echo -e "\n${COLOR_BOLD}${COLOR_CYAN}${title}${COLOR_RESET}"
    echo -e "${COLOR_DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${COLOR_RESET}"
}

##
# Image indirme satırı render eder
# Parametreler:
#   $1 - Image adı
#   $2 - Durum (downloading, extracting, complete)
#   $3 - Mevcut byte
#   $4 - Toplam byte
##
render_image_line() {
    local image=$1
    local status=$2
    local current=$3
    local total=$4
    
    # Image adını 25 karaktere sınırla
    local image_display=$(printf "%-25s" "${image:0:25}")
    
    case "$status" in
        downloading)
            local bar=$(create_progress_bar "$current" "$total" 20)
            local percent=0
            if [ "$total" -gt 0 ]; then
                percent=$((current * 100 / total))
            fi
            local current_fmt=$(format_bytes "$current")
            local total_fmt=$(format_bytes "$total")
            echo -e "${CHAR_DOWNLOAD} ${COLOR_YELLOW}${image_display}${COLOR_RESET} ${bar} ${COLOR_BOLD}${percent}%${COLOR_RESET} ${COLOR_DIM}(${current_fmt}/${total_fmt})${COLOR_RESET}"
            ;;
        extracting)
            local bar=$(create_progress_bar "$current" "$total" 20)
            local percent=0
            if [ "$total" -gt 0 ]; then
                percent=$((current * 100 / total))
            fi
            echo -e "${CHAR_PACKAGE} ${COLOR_CYAN}${image_display}${COLOR_RESET} ${bar} ${COLOR_BOLD}${percent}%${COLOR_RESET} ${COLOR_DIM}(Çıkartılıyor)${COLOR_RESET}"
            ;;
        complete)
            local bar=$(create_progress_bar 100 100 20)
            echo -e "${CHAR_CHECKMARK} ${COLOR_GREEN}${image_display}${COLOR_RESET} ${bar} ${COLOR_BOLD}100%${COLOR_RESET} ${COLOR_DIM}(Hazır)${COLOR_RESET}"
            ;;
        *)
            echo -e "${COLOR_DIM}${image_display}${COLOR_RESET} ${COLOR_DIM}Bekleniyor...${COLOR_RESET}"
            ;;
    esac
}

##
# Build satırı render eder
# Parametreler:
#   $1 - Service adı
#   $2 - Mevcut adım
#   $3 - Toplam adım
#   $4 - Adım açıklaması
##
render_build_line() {
    local service=$1
    local current_step=$2
    local total_steps=$3
    local step_desc=$4
    
    # Service adını 25 karaktere sınırla
    local service_display=$(printf "%-25s" "${service:0:25}")
    
    if [ "$current_step" -eq "$total_steps" ]; then
        echo -e "${CHAR_CHECKMARK} ${COLOR_GREEN}${service_display}${COLOR_RESET} ${COLOR_BOLD}Build tamamlandı${COLOR_RESET}"
    else
        local bar=$(create_progress_bar "$current_step" "$total_steps" 20)
        echo -e "${CHAR_GEAR} ${COLOR_BLUE}${service_display}${COLOR_RESET} ${bar} ${COLOR_DIM}[${current_step}/${total_steps}] ${step_desc}${COLOR_RESET}"
    fi
}

##
# Container satırı render eder
# Parametreler:
#   $1 - Container adı
#   $2 - Durum
##
render_container_line() {
    local container=$1
    local status=$2
    
    # Container adını 35 karaktere sınırla (daha uzun olabilir)
    local container_display=$(printf "%-35s" "${container:0:35}")
    
    case "$status" in
        Started)
            echo -e "${CHAR_CHECKMARK} ${COLOR_GREEN}${container_display}${COLOR_RESET} ${COLOR_BOLD}Hazır${COLOR_RESET}"
            ;;
        Starting)
            echo -e "${CHAR_ROCKET} ${COLOR_YELLOW}${container_display}${COLOR_RESET} ${COLOR_DIM}Başlatılıyor...${COLOR_RESET}"
            ;;
        Created)
            echo -e "${CHAR_PACKAGE} ${COLOR_CYAN}${container_display}${COLOR_RESET} ${COLOR_DIM}Oluşturuldu${COLOR_RESET}"
            ;;
        *)
            echo -e "${CHAR_GEAR} ${COLOR_DIM}${container_display}${COLOR_RESET} ${COLOR_DIM}${status}${COLOR_RESET}"
            ;;
    esac
}

##
# Footer (süre bilgisi) render eder
# Parametreler:
#   $1 - Başlangıç zamanı (epoch)
##
render_footer() {
    local start_time=$1
    local current_time=$(date +%s)
    local elapsed=$((current_time - start_time))
    local duration=$(format_duration "$elapsed")
    
    echo -e "\n${CHAR_CLOCK}  ${COLOR_DIM}Geçen Süre: ${COLOR_RESET}${COLOR_BOLD}${duration}${COLOR_RESET}"
}
