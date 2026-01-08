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

# Get new version number (as parameter)
NEW_VERSION=${1:-"Unreleased"}
RELEASE_DATE=$(date +%Y-%m-%d)

# Get previous tag (before the new version)
# If we're on a tag, get the previous one
CURRENT_TAG=$(git describe --tags --exact-match 2>/dev/null || echo "")
if [ -n "$CURRENT_TAG" ]; then
    # We're on a tag, get the previous one
    LATEST_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
else
    # We're not on a tag, get the latest one
    LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
fi

if [ -z "$LATEST_TAG" ]; then
    echo -e "${YELLOW}⚠️  First release, all commits will be used${NC}"
    COMMIT_RANGE="HEAD"
else
    echo -e "${GREEN}📌 Previous tag: $LATEST_TAG${NC}"
    echo -e "${GREEN}📌 New version: $NEW_VERSION${NC}"
    COMMIT_RANGE="$LATEST_TAG..HEAD"
fi

# Temporary files for detailed changelogs (MkDocs)
TEMP_FILE_DOCS_TR=$(mktemp)
TEMP_FILE_DOCS_EN=$(mktemp)

# Temporary file for root CHANGELOG.md (simple, version list only)
TEMP_FILE_ROOT=$(mktemp)

# Root CHANGELOG.md header (simple, English only)
cat > "$TEMP_FILE_ROOT" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

EOF

# docs/tr/changelog.md header (detailed, Turkish titles, English content, MkDocs format)
cat > "$TEMP_FILE_DOCS_TR" << EOF
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

# docs/en/changelog.md header (detailed, English content, MkDocs format)
cat > "$TEMP_FILE_DOCS_EN" << EOF
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
    echo "## [Unreleased]" >> "$TEMP_FILE_ROOT"
    echo "## [Unreleased]" >> "$TEMP_FILE_DOCS_TR"
    echo "## [Unreleased]" >> "$TEMP_FILE_DOCS_EN"
else
    echo "## [$NEW_VERSION] - $RELEASE_DATE" >> "$TEMP_FILE_ROOT"
    echo "## [$NEW_VERSION] - $RELEASE_DATE" >> "$TEMP_FILE_DOCS_TR"
    echo "## [$NEW_VERSION] - $RELEASE_DATE" >> "$TEMP_FILE_DOCS_EN"
fi

echo "" >> "$TEMP_FILE_ROOT"
echo "" >> "$TEMP_FILE_DOCS_TR"
echo "" >> "$TEMP_FILE_DOCS_EN"

# Function returning category headers (English only)
get_category() {
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

# Collect commits for each category (English only)
for type in feat fix docs style refactor perf test chore; do
    commits=$(git log $COMMIT_RANGE --pretty=format:"%s" --grep="^$type:" --no-merges 2>/dev/null || echo "")
    
    if [ -n "$commits" ]; then
        category=$(get_category $type)
        echo "$category" >> "$TEMP_FILE_ROOT"
        echo "$category" >> "$TEMP_FILE_DOCS_TR"
        echo "$category" >> "$TEMP_FILE_DOCS_EN"
        
        while IFS= read -r commit; do
            # Parse commit message
            # Format: type(scope): message
            message=$(echo "$commit" | sed -E 's/^[a-z]+(\([^)]+\))?: //')
            scope=$(echo "$commit" | sed -nE 's/^[a-z]+\(([^)]+)\):.*/\1/p')
            
            if [ -n "$scope" ]; then
                echo "- **$scope**: $message" >> "$TEMP_FILE_ROOT"
                echo "- **$scope**: $message" >> "$TEMP_FILE_DOCS_TR"
                echo "- **$scope**: $message" >> "$TEMP_FILE_DOCS_EN"
            else
                echo "- $message" >> "$TEMP_FILE_ROOT"
                echo "- $message" >> "$TEMP_FILE_DOCS_TR"
                echo "- $message" >> "$TEMP_FILE_DOCS_EN"
            fi
        done <<< "$commits"
        
        echo "" >> "$TEMP_FILE_ROOT"
        echo "" >> "$TEMP_FILE_DOCS_TR"
        echo "" >> "$TEMP_FILE_DOCS_EN"
    fi
done

# Breaking changes (English only)
breaking=$(git log $COMMIT_RANGE --pretty=format:"%s%n%b" --grep="BREAKING CHANGE:" --no-merges 2>/dev/null || echo "")
if [ -n "$breaking" ]; then
    echo "### ⚠️ BREAKING CHANGES" >> "$TEMP_FILE_ROOT"
    echo "### ⚠️ BREAKING CHANGES" >> "$TEMP_FILE_DOCS_TR"
    echo "### ⚠️ BREAKING CHANGES" >> "$TEMP_FILE_DOCS_EN"
    echo "" >> "$TEMP_FILE_ROOT"
    echo "" >> "$TEMP_FILE_DOCS_TR"
    echo "" >> "$TEMP_FILE_DOCS_EN"
    breaking_content=$(echo "$breaking" | grep -A 10 "BREAKING CHANGE" | sed 's/BREAKING CHANGE: /- /')
    echo "$breaking_content" >> "$TEMP_FILE_ROOT"
    echo "$breaking_content" >> "$TEMP_FILE_DOCS_TR"
    echo "$breaking_content" >> "$TEMP_FILE_DOCS_EN"
    echo "" >> "$TEMP_FILE_ROOT"
    echo "" >> "$TEMP_FILE_DOCS_TR"
    echo "" >> "$TEMP_FILE_DOCS_EN"
fi

echo "---" >> "$TEMP_FILE_ROOT"
echo "" >> "$TEMP_FILE_ROOT"
echo "---" >> "$TEMP_FILE_DOCS_TR"
echo "" >> "$TEMP_FILE_DOCS_TR"
echo "---" >> "$TEMP_FILE_DOCS_EN"
echo "" >> "$TEMP_FILE_DOCS_EN"

# Append old changelog (if exists) - Root CHANGELOG.md
if [ -f "CHANGELOG.md" ]; then
    # Get old content - only versioned sections (in ## [x.x.x] format)
    # Skip the new version
    OLD_CONTENT_ROOT=$(sed -n '/^## \[[0-9]/,/^---$/p' CHANGELOG.md | grep -v "^## \[$NEW_VERSION\]" || true)
    if [ -n "$OLD_CONTENT_ROOT" ]; then
        echo "$OLD_CONTENT_ROOT" >> "$TEMP_FILE_ROOT"
    fi
fi

# Append old changelog (if exists) - docs/tr/changelog.md
if [ -f "docs/tr/changelog.md" ]; then
    # Get old content - only versioned sections (in ## [x.x.x] format)
    # Skip sections like Bağlantılar and the new version
    OLD_CONTENT_DOCS_TR=$(sed -n '/^## \[[0-9]/,/^## Bağlantılar/p' docs/tr/changelog.md | grep -v "^## Bağlantılar" | grep -v "^## \[$NEW_VERSION\]" || true)
    if [ -n "$OLD_CONTENT_DOCS_TR" ]; then
        echo "$OLD_CONTENT_DOCS_TR" >> "$TEMP_FILE_DOCS_TR"
    fi
fi

# Append old changelog (if exists) - docs/en/changelog.md
if [ -f "docs/en/changelog.md" ]; then
    # Get old content - only versioned sections (in ## [x.x.x] format)
    # Skip sections like Links and the new version
    OLD_CONTENT_DOCS_EN=$(sed -n '/^## \[[0-9]/,/^## Links/p' docs/en/changelog.md | grep -v "^## Links" | grep -v "^## \[$NEW_VERSION\]" || true)
    if [ -n "$OLD_CONTENT_DOCS_EN" ]; then
        echo "$OLD_CONTENT_DOCS_EN" >> "$TEMP_FILE_DOCS_EN"
    fi
fi

# Add links to docs changelogs only (not root CHANGELOG.md)
echo "" >> "$TEMP_FILE_DOCS_TR"
echo "---" >> "$TEMP_FILE_DOCS_TR"
echo "" >> "$TEMP_FILE_DOCS_TR"
echo "## Bağlantılar" >> "$TEMP_FILE_DOCS_TR"
echo "" >> "$TEMP_FILE_DOCS_TR"
echo "- [GitHub Repository](https://github.com/stackvo/stackvo)" >> "$TEMP_FILE_DOCS_TR"
echo "- [Dokümantasyon](https://stackvo.github.io/stackvo/)" >> "$TEMP_FILE_DOCS_TR"
echo "- [Sorunlar](https://github.com/stackvo/stackvo/issues)" >> "$TEMP_FILE_DOCS_TR"
echo "- [Sürümler](https://github.com/stackvo/stackvo/releases)" >> "$TEMP_FILE_DOCS_TR"

echo "" >> "$TEMP_FILE_DOCS_EN"
echo "---" >> "$TEMP_FILE_DOCS_EN"
echo "" >> "$TEMP_FILE_DOCS_EN"
echo "## Links" >> "$TEMP_FILE_DOCS_EN"
echo "" >> "$TEMP_FILE_DOCS_EN"
echo "- [GitHub Repository](https://github.com/stackvo/stackvo)" >> "$TEMP_FILE_DOCS_EN"
echo "- [Documentation](https://stackvo.github.io/stackvo/)" >> "$TEMP_FILE_DOCS_EN"
echo "- [Issues](https://github.com/stackvo/stackvo/issues)" >> "$TEMP_FILE_DOCS_EN"
echo "- [Releases](https://github.com/stackvo/stackvo/releases)" >> "$TEMP_FILE_DOCS_EN"

# Copy new files
mkdir -p docs/tr docs/en
mv "$TEMP_FILE_ROOT" CHANGELOG.md
mv "$TEMP_FILE_DOCS_TR" docs/tr/changelog.md
mv "$TEMP_FILE_DOCS_EN" docs/en/changelog.md

echo -e "${GREEN}✅ Changelog updated:${NC}"
echo -e "${GREEN}   - CHANGELOG.md${NC}"
echo -e "${GREEN}   - docs/tr/changelog.md${NC}"
echo -e "${GREEN}   - docs/en/changelog.md${NC}"

# Statistics
echo ""
echo -e "${YELLOW}📊 Statistics:${NC}"
echo "  - Total commits: $(git log $COMMIT_RANGE --oneline --no-merges 2>/dev/null | wc -l)"
echo "  - feat: $(git log $COMMIT_RANGE --oneline --grep="^feat" --no-merges 2>/dev/null | wc -l)"
echo "  - fix: $(git log $COMMIT_RANGE --oneline --grep="^fix" --no-merges 2>/dev/null | wc -l)"
echo "  - docs: $(git log $COMMIT_RANGE --oneline --grep="^docs" --no-merges 2>/dev/null | wc -l)"
