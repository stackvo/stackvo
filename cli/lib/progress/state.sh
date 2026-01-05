#!/usr/bin/env bash

##
# State Manager - Bash 3.2 uyumlu versiyon
# Associative array yerine normal array + string manipulation
##

# State dizileri (normal array)
STATE_IMAGES=()
STATE_BUILDS=()

# Maksimum entry sayısı
MAX_IMAGES=20
MAX_BUILDS=10

# Son state hash
LAST_STATE_HASH=""

##
# State entry oluşturur
# Format: "name|status|current|total|desc"
##
create_state_entry() {
    local name=$1
    local status=$2
    local current=$3
    local total=$4
    local desc=$5
    echo "${name}|${status}|${current}|${total}|${desc}"
}

##
# State entry'den değer çıkarır
##
get_state_field() {
    local entry=$1
    local field=$2
    echo "$entry" | cut -d'|' -f"$field"
}

##
# Image durumunu günceller
##
update_image_status() {
    local image=$1
    local status=$2
    local current=${3:-0}
    local total=${4:-0}
    
    local entry=$(create_state_entry "$image" "$status" "$current" "$total" "")
    
    # Mevcut entry'yi bul ve güncelle
    local found=0
    local i=0
    for state in "${STATE_IMAGES[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" = "$image" ]; then
            STATE_IMAGES[$i]="$entry"
            found=1
            break
        fi
        ((i++))
    done
    
    # Yoksa ekle
    if [ $found -eq 0 ]; then
        STATE_IMAGES+=("$entry")
        
        # Limit kontrolü
        if [ ${#STATE_IMAGES[@]} -gt $MAX_IMAGES ]; then
            cleanup_completed_images
        fi
    fi
    
    # Complete olan image'ları 5 saniye sonra sil
    if [ "$status" = "complete" ]; then
        (sleep 5 && remove_image "$image") &
    fi
}

##
# Build durumunu günceller
##
update_build_status() {
    local service=$1
    local current=$2
    local total=$3
    local desc=$4
    
    # Açıklamayı kısalt
    local short_desc="${desc:0:30}"
    
    local entry=$(create_state_entry "$service" "building" "$current" "$total" "$short_desc")
    
    # Mevcut entry'yi bul ve güncelle
    local found=0
    local i=0
    for state in "${STATE_BUILDS[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" = "$service" ]; then
            STATE_BUILDS[$i]="$entry"
            found=1
            break
        fi
        ((i++))
    done
    
    # Yoksa ekle
    if [ $found -eq 0 ]; then
        STATE_BUILDS+=("$entry")
        
        if [ ${#STATE_BUILDS[@]} -gt $MAX_BUILDS ]; then
            cleanup_completed_builds
        fi
    fi
}

##
# Build'i tamamlandı olarak işaretle
##
complete_build() {
    local service=$1
    local entry=$(create_state_entry "$service" "complete" "5" "5" "Tamamlandı")
    
    # Güncelle
    local i=0
    for state in "${STATE_BUILDS[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" = "$service" ]; then
            STATE_BUILDS[$i]="$entry"
            break
        fi
        ((i++))
    done
    
    # 5 saniye sonra sil
    (sleep 5 && remove_build "$service") &
}

##
# Image'ı state'den sil
##
remove_image() {
    local image=$1
    local new_array=()
    for state in "${STATE_IMAGES[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" != "$image" ]; then
            new_array+=("$state")
        fi
    done
    STATE_IMAGES=("${new_array[@]}")
}

##
# Build'i state'den sil
##
remove_build() {
    local service=$1
    local new_array=()
    for state in "${STATE_BUILDS[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" != "$service" ]; then
            new_array+=("$state")
        fi
    done
    STATE_BUILDS=("${new_array[@]}")
}

##
# Tamamlanan image'ları temizle
##
cleanup_completed_images() {
    local new_array=()
    for state in "${STATE_IMAGES[@]}"; do
        local status=$(get_state_field "$state" 2)
        if [ "$status" != "complete" ]; then
            new_array+=("$state")
        fi
    done
    STATE_IMAGES=("${new_array[@]}")
}

##
# Tamamlanan build'leri temizle
##
cleanup_completed_builds() {
    local new_array=()
    for state in "${STATE_BUILDS[@]}"; do
        local status=$(get_state_field "$state" 2)
        if [ "$status" != "complete" ]; then
            new_array+=("$state")
        fi
    done
    STATE_BUILDS=("${new_array[@]}")
}

##
# Tüm image'ları listeler
##
list_images() {
    for state in "${STATE_IMAGES[@]}"; do
        get_state_field "$state" 1
    done | sort
}

##
# Tüm build'leri listeler
##
list_builds() {
    for state in "${STATE_BUILDS[@]}"; do
        get_state_field "$state" 1
    done | sort
}

##
# Image bilgilerini döndürür
# Çıktı: "status current total"
##
get_image_info() {
    local image=$1
    
    for state in "${STATE_IMAGES[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" = "$image" ]; then
            local status=$(get_state_field "$state" 2)
            local current=$(get_state_field "$state" 3)
            local total=$(get_state_field "$state" 4)
            echo "$status $current $total"
            return
        fi
    done
    
    # Bulunamadı
    echo "pending 0 0"
}

##
# Build bilgilerini döndürür
# Çıktı: "current total description"
##
get_build_info() {
    local service=$1
    
    for state in "${STATE_BUILDS[@]}"; do
        local name=$(get_state_field "$state" 1)
        if [ "$name" = "$service" ]; then
            local current=$(get_state_field "$state" 3)
            local total=$(get_state_field "$state" 4)
            local desc=$(get_state_field "$state" 5)
            echo "$current $total $desc"
            return
        fi
    done
    
    # Bulunamadı
    echo "0 5 Başlatılıyor"
}

##
# State hash hesaplar
##
get_state_hash() {
    local hash=""
    hash+="${#STATE_IMAGES[@]}"
    hash+="${#STATE_BUILDS[@]}"
    for state in "${STATE_IMAGES[@]}"; do
        hash+="$state"
    done
    for state in "${STATE_BUILDS[@]}"; do
        hash+="$state"
    done
    echo "$hash"
}
