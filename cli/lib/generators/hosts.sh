#!/bin/bash
###################################################################
# STACKVO HOSTS FILE GENERATOR MODULE
# Automatic /etc/hosts management
###################################################################

##
# Detect operating system
#
# Returns:
#   "wsl" - Windows Subsystem for Linux
#   "macos" - macOS
#   "linux" - Linux
##
detect_os() {
    if grep -qi microsoft /proc/version 2>/dev/null; then
        echo "wsl"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "linux"
    fi
}

##
# Get Windows hosts file path (WSL only)
#
# Returns:
#   Path to Windows hosts file or empty string
##
get_windows_hosts_path() {
    local os_type=$(detect_os)
    if [ "$os_type" = "wsl" ]; then
        echo "/mnt/c/Windows/System32/drivers/etc/hosts"
    fi
}

##
# Collect all domains from .env and projects
#
# Returns:
#   List of unique domains (one per line)
##
collect_domains() {
    local domains=()
    
    # 1. Traefik dashboard
    domains+=("traefik.stackvo.loc")
    
    # 2. Services - .env'den SERVICE_*_URL oku
    if [ -f "$ROOT_DIR/.env" ]; then
        while IFS='=' read -r key value; do
            # Skip comments and empty lines
            [[ "$key" =~ ^#.*$ || -z "$key" ]] && continue
            
            if [[ $key =~ ^SERVICE_([A-Z0-9_]+)_URL$ ]]; then
                local service_name="${BASH_REMATCH[1]}"
                local service_enable_var="SERVICE_${service_name}_ENABLE"
                eval "local enabled=\${${service_enable_var}:-false}"
                
                if [ "$enabled" = "true" ]; then
                    # URL'den domain çıkar
                    local domain=$(echo "$value" | sed 's|https\?://||' | cut -d'/' -f1)
                    if [[ ! "$domain" =~ \. ]]; then
                        domain="${domain}.stackvo.loc"
                    fi
                    domains+=("$domain")
                fi
            fi
        done < "$ROOT_DIR/.env"
    fi
    
    # 3. Tools - .env'den TOOLS_*_URL oku
    if [ -f "$ROOT_DIR/.env" ]; then
        while IFS='=' read -r key value; do
            # Skip comments and empty lines
            [[ "$key" =~ ^#.*$ || -z "$key" ]] && continue
            
            if [[ $key =~ ^TOOLS_([A-Z0-9_]+)_URL$ ]]; then
                local tool_name="${BASH_REMATCH[1]}"
                local tool_enable_var="TOOLS_${tool_name}_ENABLE"
                eval "local enabled=\${${tool_enable_var}:-false}"
                
                if [ "$enabled" = "true" ]; then
                    local domain=$(echo "$value" | sed 's|https\?://||' | cut -d'/' -f1)
                    if [[ ! "$domain" =~ \. ]]; then
                        domain="${domain}.stackvo.loc"
                    fi
                    domains+=("$domain")
                fi
            fi
        done < "$ROOT_DIR/.env"
    fi
    
    # 4. Projects - stackvo.json dosyalarından domain oku
    local projects_dir="$ROOT_DIR/projects"
    if [ -d "$projects_dir" ]; then
        for project_dir in "$projects_dir"/*/; do
            [ -d "$project_dir" ] || continue
            local config_file="${project_dir}stackvo.json"
            if [ -f "$config_file" ]; then
                # jq yoksa python kullan
                local domain=""
                if command -v jq &>/dev/null; then
                    domain=$(jq -r '.domain // empty' "$config_file" 2>/dev/null)
                elif command -v python3 &>/dev/null; then
                    domain=$(python3 -c "import json; print(json.load(open('$config_file')).get('domain', ''))" 2>/dev/null)
                fi
                
                if [ -n "$domain" ]; then
                    domains+=("$domain")
                fi
            fi
        done
    fi
    
    # Unique domains
    printf '%s\n' "${domains[@]}" | sort -u
}

##
# Update hosts file
#
# Parameters:
#   $1 - Hosts file path
#   $@ - List of domains
##
update_hosts_file() {
    local hosts_file=$1
    shift
    local domains=("$@")
    
    if [ ! -f "$hosts_file" ]; then
        log_warn "Hosts file not found: $hosts_file"
        return 1
    fi
    
    # Backup
    local backup_file="${hosts_file}.backup.$(date +%Y%m%d_%H%M%S)"
    if ! sudo cp "$hosts_file" "$backup_file" 2>/dev/null; then
        log_error "Failed to create backup: $backup_file"
        return 1
    fi
    
    # Remove old Stackvo entries (macOS/Linux compatible)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS requires empty string after -i
        sudo sed -i '' '/# Stackvo - Auto-generated/,/# Stackvo - End/d' "$hosts_file" 2>/dev/null
    else
        # Linux
        sudo sed -i '/# Stackvo - Auto-generated/,/# Stackvo - End/d' "$hosts_file" 2>/dev/null
    fi
    
    # Add new entries
    {
        echo ""
        echo "# Stackvo - Auto-generated - DO NOT EDIT MANUALLY"
        for domain in "${domains[@]}"; do
            echo "127.0.0.1       $domain"
        done
        echo "# Stackvo - End"
    } | sudo tee -a "$hosts_file" > /dev/null
    
    
    log_success "Updated $hosts_file (${#domains[@]} domains)"
}

##
# Generate hosts entries
#
# Returns:
#   0 - Success
#   1 - Error
##
generate_hosts() {
    log_info "Updating hosts file..."
    
    local os_type=$(detect_os)
    
    # Collect domains into array (Bash 3.x compatible)
    local domains=()
    while IFS= read -r domain; do
        [ -n "$domain" ] && domains+=("$domain")
    done < <(collect_domains)
    
    if [ ${#domains[@]} -eq 0 ]; then
        log_warn "No domains found, skipping hosts update"
        return 0
    fi
    
    log_info "Found ${#domains[@]} domains to add"
    
    # Update Linux/macOS hosts
    if update_hosts_file "/etc/hosts" "${domains[@]}"; then
        log_success "Linux/macOS hosts file updated"
    else
        log_error "Failed to update Linux/macOS hosts file"
        return 1
    fi
    
    # Update Windows hosts (WSL only)
    if [ "$os_type" = "wsl" ]; then
        local win_hosts=$(get_windows_hosts_path)
        if [ -n "$win_hosts" ] && [ -f "$win_hosts" ]; then
            log_info "Updating Windows hosts file..."
            if update_hosts_file "$win_hosts" "${domains[@]}"; then
                log_success "Windows hosts file updated"
            else
                log_warn "Failed to update Windows hosts file (may need Administrator rights)"
            fi
        else
            log_warn "Windows hosts file not found: $win_hosts"
        fi
    fi
    
    log_success "Hosts file management completed"
}
