#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/state.sh"
source "$SCRIPT_DIR/parser.sh"
source "$SCRIPT_DIR/renderer.sh"

##
# Ana progress wrapper - OPTIMIZE EDİLMİŞ
##

# Global değişkenler
START_TIME=$(date +%s)
FIRST_RENDER=true
LAST_LINE_COUNT=0

# OPTİMİZASYON: Akıllı refresh interval
REFRESH_INTERVAL=0  # 0.5 saniye yerine her render'da kontrol et (daha dinamik)
MIN_REFRESH_INTERVAL=0
MAX_REFRESH_INTERVAL=1

##
# Ekranı render eder - OPTIMIZE EDİLMİŞ
##
render_screen() {
    # OPTİMİZASYON: Sadece değişiklik varsa render et
    local current_hash=$(get_state_hash)
    if [ "$current_hash" = "$LAST_STATE_HASH" ]; then
        return  # Değişiklik yok, render etme
    fi
    LAST_STATE_HASH="$current_hash"
    
    # İlk render: Başlık göster
    if [ "$FIRST_RENDER" = true ]; then
        echo -e "\n${CHAR_ROCKET} ${COLOR_BOLD}${COLOR_GREEN}Stackvo Başlatılıyor${COLOR_RESET} ${COLOR_DIM}(minimal mod)${COLOR_RESET}\n"
        FIRST_RENDER=false
    else
        # Sonraki render'lar: Cursor'ı yukarı taşı
        if [ "$LAST_LINE_COUNT" -gt 0 ]; then
            # Önceki çıktıyı temizle (cursor yukarı + satır sil)
            for ((i=0; i<LAST_LINE_COUNT; i++)); do
                printf '\033[1A\033[2K'  # Cursor yukarı + satır sil
            done
        fi
    fi
    
    local line_count=0
    
    # Image indirme durumu
    local images=$(list_images)
    if [ -n "$images" ]; then
        render_header "${CHAR_PACKAGE} IMAGE İNDİRME DURUMU"
        ((line_count+=3))  # Başlık + çizgi + boş satır
        while IFS= read -r image; do
            local info=$(get_image_info "$image")
            local status=$(echo "$info" | awk '{print $1}')
            local current=$(echo "$info" | awk '{print $2}')
            local total=$(echo "$info" | awk '{print $3}')
            render_image_line "$image" "$status" "$current" "$total"
            ((line_count++))
        done <<< "$images"
    fi
    
    # Build durumu
    local builds=$(list_builds)
    if [ -n "$builds" ]; then
        render_header "${CHAR_BUILD} BUILD DURUMU"
        ((line_count+=3))  # Başlık + çizgi + boş satır
        while IFS= read -r service; do
            local info=$(get_build_info "$service")
            local current=$(echo "$info" | awk '{print $1}')
            local total=$(echo "$info" | awk '{print $2}')
            local desc=$(echo "$info" | cut -d' ' -f3-)
            render_build_line "$service" "$current" "$total" "$desc"
            ((line_count++))
        done <<< "$builds"
    fi

    # Container durumu
    local containers=$(list_containers)
    if [ -n "$containers" ]; then
        render_header "${CHAR_ROCKET} CONTAINER DURUMU"
        ((line_count+=3))
        while IFS= read -r container; do
            local status=$(get_container_info "$container")
            render_container_line "$container" "$status"
            ((line_count++))
        done <<< "$containers"
    fi
    
    # Footer
    render_footer "$START_TIME"
    ((line_count+=2))  # Footer 2 satır
    
    LAST_LINE_COUNT=$line_count
}

##
# Docker Compose çıktısını okur ve progress gösterir - TIMEOUT MEKANİZMASI İLE
##
show_docker_progress() {
    local last_render=0
    local line_count=0
    
    while true; do
        # Timeout ile satır oku (1 saniye)
        if IFS= read -r -t 1 line; then
            # Satır geldi, parse et
            parse_docker_line "$line"
            
            ((line_count++))
            
            # OPTİMİZASYON: Bash built-in ile zaman kontrolü
            local current_time=$SECONDS
            local elapsed=$((current_time - last_render))
            
            # Akıllı refresh stratejisi
            local should_render=0
            
            # Strateji 1: Her 50 satırda bir render et
            if [ $((line_count % 50)) -eq 0 ]; then
                should_render=1
            fi
            
            # Strateji 2: Minimum interval geçtiyse render et
            if [ "$elapsed" -ge "$REFRESH_INTERVAL" ]; then
                should_render=1
            fi
            
            # Strateji 3: Önemli olaylar anında render et
            if [[ "$line" =~ (Pull\ complete|Built|Started|Pulled|Creating|Created) ]]; then
                should_render=1
            fi
            
            if [ "$should_render" -eq 1 ]; then
                render_screen
                last_render=$current_time
                line_count=0
            fi
        else
            # Timeout oldu (1 saniye boyunca satır gelmedi)
            local read_exit=$?
            
            # read exit code kontrolü:
            # 0 = başarılı (satır okundu)
            # 1 = EOF (pipe kapandı - Docker Compose bitti)
            # >128 = timeout
            
            if [ "$read_exit" -eq 1 ]; then
                # EOF - Docker Compose bitti, çık
                break
            fi
            
            # Timeout - sadece render yap, devam et
            render_screen
            last_render=$SECONDS
        fi
    done
    
    # Son render
    render_screen
}
