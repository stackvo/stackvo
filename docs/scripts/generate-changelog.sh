#!/bin/bash

# Automatic Changelog Generator
# Generates changelog from Conventional Commits

set -e

# Determine script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Move to project root
cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Changelog Generator${NC}"

# Get latest tag
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LATEST_TAG" ]; then
    echo -e "${YELLOW}⚠️  First release, all commits will be used${NC}"
    COMMIT_RANGE="HEAD"
else
    echo -e "${GREEN}📌 Latest tag: $LATEST_TAG${NC}"
    COMMIT_RANGE="$LATEST_TAG..HEAD"
fi

# Get new version number (as parameter)
NEW_VERSION=${1:-"Unreleased"}
RELEASE_DATE=$(date +%Y-%m-%d)

# Temporary files
TEMP_FILE_TR=$(mktemp)
TEMP_FILE_EN=$(mktemp)

# Turkish Changelog header
cat > "$TEMP_FILE_TR" << EOF
---
title: Değişiklikler
description: Stackvo projesindeki tüm önemli değişiklikler bu dosyada dokümante edilir. Bu sayfa, her versiyonda eklenen yeni özellikler, değiştirilen fonksiyonlar, düzeltilen hatalar ve güvenlik güncellemelerini içermektedir. Semantic Versioning ve Keep a Changelog standartlarına uygun olarak düzenlenmektedir.
hide:
  - navigation
---

# Değişiklikler

Stackvo projesindeki tüm önemli değişiklikler bu dosyada dokümante edilir.

Format [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) standardına dayanır ve proje [Semantic Versioning](https://semver.org/spec/v2.0.0.html) kullanır.

---

EOF

# English Changelog header
cat > "$TEMP_FILE_EN" << EOF
---
title: Changelog
description: All notable changes to the Stackvo project are documented in this file. This page includes new features, changed functions, bug fixes, and security updates for each version. It follows the Semantic Versioning and Keep a Changelog standards.
hide:
  - navigation
---

# Changelog

All notable changes to the Stackvo project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

EOF

# New version header
if [ "$NEW_VERSION" = "Unreleased" ]; then
    echo "## [Unreleased]" >> "$TEMP_FILE_TR"
    echo "## [Unreleased]" >> "$TEMP_FILE_EN"
else
    echo "## [$NEW_VERSION] - $RELEASE_DATE" >> "$TEMP_FILE_TR"
    echo "## [$NEW_VERSION] - $RELEASE_DATE" >> "$TEMP_FILE_EN"
fi

echo "" >> "$TEMP_FILE_TR"
echo "" >> "$TEMP_FILE_EN"

# Functions returning category headers
get_category_tr() {
    case "$1" in
        feat) echo "### Eklenenler" ;;
        fix) echo "### Düzeltmeler" ;;
        docs) echo "### Dokümantasyon" ;;
        style) echo "### Stil" ;;
        refactor) echo "### Yeniden Yapılandırma" ;;
        perf) echo "### Performans" ;;
        test) echo "### Testler" ;;
        chore) echo "### Diğer" ;;
    esac
}

get_category_en() {
    case "$1" in
        feat) echo "### Added" ;;
        fix) echo "### Fixed" ;;
        docs) echo "### Documentation" ;;
        style) echo "### Style" ;;
        refactor) echo "### Refactored" ;;
        perf) echo "### Performance" ;;
        test) echo "### Tests" ;;
        chore) echo "### Chore" ;;
    esac
}

# Collect commits for each category
for type in feat fix docs style refactor perf test chore; do
    commits=$(git log $COMMIT_RANGE --pretty=format:"%s" --grep="^$type" --no-merges 2>/dev/null || echo "")
    
    if [ -n "$commits" ]; then
        echo "$(get_category_tr $type)" >> "$TEMP_FILE_TR"
        echo "$(get_category_en $type)" >> "$TEMP_FILE_EN"
        
        while IFS= read -r commit; do
            # Parse commit message
            # Format: type(scope): message
            message=$(echo "$commit" | sed -E 's/^[a-z]+(\([^)]+\))?: //')
            scope=$(echo "$commit" | sed -nE 's/^[a-z]+\(([^)]+)\):.*/\1/p')
            
            if [ -n "$scope" ]; then
                echo "- **$scope**: $message" >> "$TEMP_FILE_TR"
                echo "- **$scope**: $message" >> "$TEMP_FILE_EN"
            else
                echo "- $message" >> "$TEMP_FILE_TR"
                echo "- $message" >> "$TEMP_FILE_EN"
            fi
        done <<< "$commits"
        
        echo "" >> "$TEMP_FILE_TR"
        echo "" >> "$TEMP_FILE_EN"
    fi
done

# Breaking changes
breaking=$(git log $COMMIT_RANGE --pretty=format:"%s%n%b" --grep="BREAKING CHANGE" --no-merges 2>/dev/null || echo "")
if [ -n "$breaking" ]; then
    echo "### ⚠️ KIRILAMAYAN DEĞİŞİKLİKLER" >> "$TEMP_FILE_TR"
    echo "### ⚠️ BREAKING CHANGES" >> "$TEMP_FILE_EN"
    echo "" >> "$TEMP_FILE_TR"
    echo "" >> "$TEMP_FILE_EN"
    echo "$breaking" | grep -A 10 "BREAKING CHANGE" | sed 's/BREAKING CHANGE: /- /' >> "$TEMP_FILE_TR"
    echo "$breaking" | grep -A 10 "BREAKING CHANGE" | sed 's/BREAKING CHANGE: /- /' >> "$TEMP_FILE_EN"
    echo "" >> "$TEMP_FILE_TR"
    echo "" >> "$TEMP_FILE_EN"
fi

echo "---" >> "$TEMP_FILE_TR"
echo "" >> "$TEMP_FILE_TR"
echo "---" >> "$TEMP_FILE_EN"
echo "" >> "$TEMP_FILE_EN"

# Append old changelog (if exists) - Turkish
if [ -f "docs/tr/changelog.md" ]; then
    # Get old content - only versioned sections (in ## [x.x.x] format)
    # Skip sections like Unreleased, Planned, Links
    OLD_CONTENT_TR=$(sed -n '/^## \[[0-9]/,/^## Bağlantılar/p' docs/tr/changelog.md | grep -v "^## Bağlantılar" || true)
    if [ -n "$OLD_CONTENT_TR" ]; then
        echo "$OLD_CONTENT_TR" >> "$TEMP_FILE_TR"
    fi
fi

# Append old changelog (if exists) - English
if [ -f "docs/en/changelog.md" ]; then
    # Get old content - only versioned sections (in ## [x.x.x] format)
    # Skip sections like Unreleased, Planned, Links
    OLD_CONTENT_EN=$(sed -n '/^## \[[0-9]/,/^## Links/p' docs/en/changelog.md | grep -v "^## Links" || true)
    if [ -n "$OLD_CONTENT_EN" ]; then
        echo "$OLD_CONTENT_EN" >> "$TEMP_FILE_EN"
    fi
fi

# Add version links - Turkish
echo "" >> "$TEMP_FILE_TR"
echo "---" >> "$TEMP_FILE_TR"
echo "" >> "$TEMP_FILE_TR"
echo "## Bağlantılar" >> "$TEMP_FILE_TR"
echo "" >> "$TEMP_FILE_TR"
echo "- [GitHub Repository](https://github.com/stackvo/stackvo)" >> "$TEMP_FILE_TR"
echo "- [Dokümantasyon](https://stackvo.github.io/stackvo/)" >> "$TEMP_FILE_TR"
echo "- [Sorunlar](https://github.com/stackvo/stackvo/issues)" >> "$TEMP_FILE_TR"
echo "- [Sürümler](https://github.com/stackvo/stackvo/releases)" >> "$TEMP_FILE_TR"

# Add version links - English
echo "" >> "$TEMP_FILE_EN"
echo "---" >> "$TEMP_FILE_EN"
echo "" >> "$TEMP_FILE_EN"
echo "## Links" >> "$TEMP_FILE_EN"
echo "" >> "$TEMP_FILE_EN"
echo "- [GitHub Repository](https://github.com/stackvo/stackvo)" >> "$TEMP_FILE_EN"
echo "- [Documentation](https://stackvo.github.io/stackvo/)" >> "$TEMP_FILE_EN"
echo "- [Issues](https://github.com/stackvo/stackvo/issues)" >> "$TEMP_FILE_EN"
echo "- [Releases](https://github.com/stackvo/stackvo/releases)" >> "$TEMP_FILE_EN"

# Copy new files
mkdir -p docs/tr docs/en
mv "$TEMP_FILE_TR" docs/tr/changelog.md
mv "$TEMP_FILE_EN" docs/en/changelog.md

echo -e "${GREEN}✅ Changelog updated:${NC}"
echo -e "${GREEN}   - docs/tr/changelog.md${NC}"
echo -e "${GREEN}   - docs/en/changelog.md${NC}"

# Statistics
echo ""
echo -e "${YELLOW}📊 Statistics:${NC}"
echo "  - Total commits: $(git log $COMMIT_RANGE --oneline --no-merges 2>/dev/null | wc -l)"
echo "  - feat: $(git log $COMMIT_RANGE --oneline --grep="^feat" --no-merges 2>/dev/null | wc -l)"
echo "  - fix: $(git log $COMMIT_RANGE --oneline --grep="^fix" --no-merges 2>/dev/null | wc -l)"
echo "  - docs: $(git log $COMMIT_RANGE --oneline --grep="^docs" --no-merges 2>/dev/null | wc -l)"
