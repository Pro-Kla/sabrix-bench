#!/usr/bin/env bash
set -e

VERSION="v0.1.0"
NOTES_FILE="RELEASE_NOTES_v0.1.0.md"

# ANSI Color Codes
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║             SABRIX-BENCH AUTOMATED PRODUCTION RELEASE (${VERSION})               ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Step 1: Run Full Verification Suite
echo -e "\n${BOLD}${CYAN}▶ Step 1: Running Complete Verification & Stress Suite...${NC}"
./verify.sh

# Step 2: Ensure Git Working Directory is Clean
echo -e "\n${BOLD}${CYAN}▶ Step 2: Checking Git Status...${NC}"
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Uncommitted changes detected in working tree. Commit or stash them first.${NC}"
    git status -s
    exit 1
fi
echo -e "${GREEN}✔ Working tree is clean.${NC}"

# Step 3: Check / Configure Git Remote
echo -e "\n${BOLD}${CYAN}▶ Step 3: Checking Git Remote 'origin'...${NC}"
if ! git remote get-url origin >/dev/null 2>&1; then
    echo -e "${YELLOW}Remote 'origin' not configured.${NC}"
    read -p "Enter GitHub repository URL (e.g., git@github.com:sabrix-ai/sabrix-bench.git): " REMOTE_URL
    if [ -n "$REMOTE_URL" ]; then
        git remote add origin "$REMOTE_URL"
        echo -e "${GREEN}✔ Added remote origin: ${REMOTE_URL}${NC}"
    else
        echo -e "${RED}Error: GitHub remote origin is required to push release.${NC}"
        exit 1
    fi
else
    REMOTE_URL=$(git remote get-url origin)
    echo -e "${GREEN}✔ Found existing remote origin: ${REMOTE_URL}${NC}"
fi

# Step 4: Ensure on main branch and push
echo -e "\n${BOLD}${CYAN}▶ Step 4: Pushing branch 'main' to origin...${NC}"
git branch -M main
git push -u origin main
echo -e "${GREEN}✔ Pushed main branch to origin.${NC}"

# Step 5: Tag Release
echo -e "\n${BOLD}${CYAN}▶ Step 5: Creating & Pushing Annotated Git Tag (${VERSION})...${NC}"
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    echo -e "${YELLOW}Tag ${VERSION} already exists locally.${NC}"
else
    git tag -a "$VERSION" -m "Release ${VERSION}: MCP JSON-RPC Tracer & Agent Loop Latency Profiler"
    echo -e "${GREEN}✔ Created annotated tag ${VERSION}.${NC}"
fi

git push origin "$VERSION"
echo -e "${GREEN}✔ Pushed tag ${VERSION} to origin.${NC}"

# Step 6: Create GitHub Release via GitHub CLI if available
echo -e "\n${BOLD}${CYAN}▶ Step 6: Creating GitHub Release...${NC}"
if command -v gh >/dev/null 2>&1; then
    if gh auth status >/dev/null 2>&1; then
        echo "Creating GitHub Release via 'gh' CLI..."
        gh release create "$VERSION" \
            --title "${VERSION} - Initial Release: MCP Tracer & Agent Loop Latency Profiler" \
            --notes-file "$NOTES_FILE"
        echo -e "${GREEN}✔ GitHub Release published successfully!${NC}"
    else
        echo -e "${YELLOW}Notice: 'gh' CLI is installed but not authenticated. Skipping automated GitHub release creation.${NC}"
        echo "You can create the release manually on GitHub using ${NOTES_FILE}."
    fi
else
    echo -e "${YELLOW}Notice: 'gh' CLI not found. Skipping automated GitHub release creation.${NC}"
    echo "You can create the release manually on GitHub using ${NOTES_FILE}."
fi

# Step 7: Publishing Instructions for Crates.io
echo -e "\n${BOLD}${CYAN}▶ Step 7: Crates.io Publication Instructions${NC}"
echo -e "────────────────────────────────────────────────────────────────────────────────"
echo -e "To publish this release to crates.io, run:"
echo -e "\n   ${BOLD}${GREEN}cargo publish${NC}\n"
echo -e "Make sure you are logged in via 'cargo login <token>'."

echo -e "\n${GREEN}${BOLD}"
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║                 RELEASE ${VERSION} FLOW COMPLETED SUCCESSFULLY!                ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
