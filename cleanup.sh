#!/bin/bash
# KomoGuard System Cleanup
# Removes only safe caches and trash. No system files touched.

GREEN='\033[1;32m'
CYAN='\033[1;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo "  ${CYAN}Kali Linux Safe Cleanup${NC}"
echo "  ${YELLOW}Removing caches and trash only — no system files${NC}"
echo ""

clean_dir() {
    local path="$1"
    local label="$2"
    if [ -d "$path" ]; then
        local bytes=$(du -sb "$path" 2>/dev/null | cut -f1)
        local size=$(du -sh "$path" 2>/dev/null | cut -f1)
        rm -rf "$path" 2>/dev/null
        TOTAL=$((TOTAL + bytes))
        echo "  ${GREEN}[✓]${NC} $label (freed $size)"
    else
        echo "  ${YELLOW}[i]${NC} $label — nothing to clean"
    fi
}

# 1. Trash
echo "  ${CYAN}[1/5]${NC} Emptying trash..."
clean_dir "$HOME/.local/share/Trash/" "Trash"

# 2. Gradle build cache
echo "  ${CYAN}[2/5]${NC} Clearing Gradle build caches..."
clean_dir "$HOME/.gradle/caches/" "Gradle caches"
clean_dir "$HOME/.gradle/wrapper/" "Gradle wrappers"

# 3. npm cache
echo "  ${CYAN}[3/5]${NC} Clearing npm cache..."
if [ -d "$HOME/.npm/_cacache" ]; then
    n_bytes=$(du -sb "$HOME/.npm/_cacache" 2>/dev/null | cut -f1)
    n_size=$(du -sh "$HOME/.npm/_cacache" 2>/dev/null | cut -f1)
    npm cache clean --force 2>/dev/null
    TOTAL=$((TOTAL + n_bytes))
    echo "  ${GREEN}[✓]${NC} npm cache (freed $n_size)"
elif command -v npm &>/dev/null; then
    npm cache clean --force 2>/dev/null
    echo "  ${GREEN}[✓]${NC} npm cache cleared"
else
    echo "  ${YELLOW}[i]${NC} npm not installed — skipped"
fi

# 4. Thumbnails
echo "  ${CYAN}[4/5]${NC} Clearing thumbnail cache..."
clean_dir "$HOME/.cache/thumbnails/" "Thumbnails"

# 5. Browser caches
echo "  ${CYAN}[5/5]${NC} Clearing browser caches..."
clean_dir "$HOME/.cache/chromium/" "Chromium cache"

echo ""
FREED_MB=$(( TOTAL / 1024 / 1024 ))
echo "  ${GREEN}Done. Freed ~${FREED_MB}MB${NC}"
echo ""
