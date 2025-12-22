#!/bin/bash

# Release Helper Script
# Yeni versiyon oluşturur ve changelog'u otomatik günceller

set -e

# Script dizinini ve proje kök dizinini belirle
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Proje kök dizinine geç
cd "$PROJECT_ROOT"

# Renkler
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Stackvo Release Helper        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════╝${NC}"
echo ""

# Mevcut versiyonu al
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo -e "${GREEN}📌 Mevcut versiyon: $CURRENT_VERSION${NC}"
echo ""

# Versiyon tipini sor
echo -e "${YELLOW}Versiyon tipi seçin:${NC}"
echo "  1) Major (breaking changes)  - ${CURRENT_VERSION} → vX.0.0"
echo "  2) Minor (new features)      - ${CURRENT_VERSION} → v0.X.0"
echo "  3) Patch (bug fixes)         - ${CURRENT_VERSION} → v0.0.X"
echo ""
read -p "Seçiminiz (1-3): " choice

# Yeni versiyonu hesapla
CURRENT_VERSION_NUM=${CURRENT_VERSION#v}
IFS='.' read -r -a version_parts <<< "$CURRENT_VERSION_NUM"

case $choice in
    1)
        NEW_VERSION="v$((version_parts[0] + 1)).0.0"
        ;;
    2)
        NEW_VERSION="v${version_parts[0]}.$((version_parts[1] + 1)).0"
        ;;
    3)
        NEW_VERSION="v${version_parts[0]}.${version_parts[1]}.$((version_parts[2] + 1))"
        ;;
    *)
        echo -e "${RED}❌ Geçersiz seçim${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}🎯 Yeni versiyon: $NEW_VERSION${NC}"
echo ""

# Son commit'lerden beri yapılan değişiklikleri göster
echo -e "${YELLOW}📝 Son değişiklikler:${NC}"
git log $CURRENT_VERSION..HEAD --oneline --no-merges | head -10
echo ""

# Onay iste
read -p "Devam etmek istiyor musunuz? (y/n): " confirm
if [ "$confirm" != "y" ]; then
    echo -e "${RED}❌ İptal edildi${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}🔄 İşlemler başlıyor...${NC}"

# 1. Changelog oluştur
echo -e "${YELLOW}1/4 Changelog oluşturuluyor...${NC}"
./scripts/generate-changelog.sh ${NEW_VERSION#v}

# 2. Git add
echo -e "${YELLOW}2/4 Değişiklikler commit ediliyor...${NC}"
git add docs/tr/changelog.md docs/en/changelog.md
git commit -m "docs: update changelog for $NEW_VERSION"

# 3. Tag oluştur
echo -e "${YELLOW}3/4 Tag oluşturuluyor...${NC}"
git tag -a $NEW_VERSION -m "Release $NEW_VERSION"

# 4. Push
echo -e "${YELLOW}4/4 Push ediliyor...${NC}"
git push origin main
git push origin $NEW_VERSION

echo ""
echo -e "${GREEN}╔════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   ✅ Release başarılı!             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📦 Versiyon: $NEW_VERSION${NC}"
echo -e "${BLUE}🔗 GitHub Actions changelog'u otomatik güncelleyecek${NC}"
echo -e "${BLUE}🔗 GitHub Release otomatik oluşturulacak${NC}"
echo ""
echo -e "${YELLOW}Kontrol edin:${NC}"
echo "  - https://github.com/stackvo/stackvo/releases"
echo "  - https://github.com/stackvo/stackvo/actions"
