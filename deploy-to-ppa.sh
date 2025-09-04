#!/bin/bash

# Deploy Cortex to PPA via GitHub Actions

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}     Cortex PPA Deployment Tool${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Configuration
VERSION="0.1.2"
GITHUB_REPO="trinverse/cortex"
WORKFLOW="publish-ppa-all.yml"

echo -e "${YELLOW}Deployment Configuration:${NC}"
echo "  Version: $VERSION"
echo "  Repository: $GITHUB_REPO"
echo "  Workflow: $WORKFLOW"
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}GitHub CLI (gh) is not installed${NC}"
    echo "Install it with: sudo apt install gh"
    echo ""
    echo "Or use the web interface:"
    echo "https://github.com/${GITHUB_REPO}/actions/workflows/${WORKFLOW}"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}GitHub CLI not authenticated${NC}"
    echo "Run: gh auth login"
    exit 1
fi

echo -e "${GREEN}✓ GitHub CLI is ready${NC}"
echo ""

# Menu
echo -e "${YELLOW}Select deployment option:${NC}"
echo "1) Deploy to ALL stable versions (recommended)"
echo "2) Test with focal (Ubuntu 20.04) only"
echo "3) Deploy to LTS versions only"
echo "4) Deploy to all including development (plucky)"
echo "5) Deploy to a specific version"
echo "6) Check workflow status"
echo "7) View Launchpad PPA page"
read -p "Choice [1]: " CHOICE
CHOICE=${CHOICE:-1}

case $CHOICE in
    1)
        echo -e "${BLUE}Starting deployment for ALL stable versions...${NC}"
        gh workflow run ${WORKFLOW} \
            -R ${GITHUB_REPO} \
            -f version=${VERSION} \
            -f distribution=all
        echo -e "${GREEN}✓ Workflow started for all stable distributions${NC}"
        echo "Building for: focal, jammy, noble, oracular"
        echo ""
        echo "Monitor at: https://github.com/${GITHUB_REPO}/actions"
        ;;
    
    2)
        echo -e "${BLUE}Starting deployment for focal (test)...${NC}"
        gh workflow run ${WORKFLOW} \
            -R ${GITHUB_REPO} \
            -f version=${VERSION} \
            -f distribution=focal
        echo -e "${GREEN}✓ Workflow started for focal${NC}"
        echo ""
        echo "Monitor at: https://github.com/${GITHUB_REPO}/actions"
        ;;
    
    3)
        echo -e "${BLUE}Starting deployment for LTS versions only...${NC}"
        gh workflow run ${WORKFLOW} \
            -R ${GITHUB_REPO} \
            -f version=${VERSION} \
            -f distribution=lts-only
        echo -e "${GREEN}✓ Workflow started for LTS versions${NC}"
        echo "Building for: focal, jammy, noble"
        echo ""
        echo "Monitor at: https://github.com/${GITHUB_REPO}/actions"
        ;;
    
    4)
        echo -e "${BLUE}Starting deployment for ALL including development...${NC}"
        gh workflow run ${WORKFLOW} \
            -R ${GITHUB_REPO} \
            -f version=${VERSION} \
            -f distribution=all-including-dev
        echo -e "${GREEN}✓ Workflow started for all distributions${NC}"
        echo "Building for: focal, jammy, noble, oracular, plucky"
        echo ""
        echo "Monitor at: https://github.com/${GITHUB_REPO}/actions"
        ;;
    
    5)
        echo "Available distributions:"
        echo "  - focal (20.04 LTS)"
        echo "  - jammy (22.04 LTS)"
        echo "  - noble (24.04 LTS)"
        echo "  - oracular (24.10)"
        echo "  - plucky (25.04 dev)"
        read -p "Enter distribution: " DIST
        
        echo -e "${BLUE}Starting deployment for $DIST...${NC}"
        gh workflow run ${WORKFLOW} \
            -R ${GITHUB_REPO} \
            -f version=${VERSION} \
            -f distribution=$DIST
        echo -e "${GREEN}✓ Workflow started for $DIST${NC}"
        ;;
    
    6)
        echo -e "${BLUE}Recent workflow runs:${NC}"
        gh run list \
            -R ${GITHUB_REPO} \
            --workflow=${WORKFLOW} \
            --limit 5
        ;;
    
    7)
        echo -e "${BLUE}Opening Launchpad PPA page...${NC}"
        xdg-open "https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex" 2>/dev/null || \
            echo "Visit: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex"
        ;;
esac

echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Monitor GitHub Actions: https://github.com/${GITHUB_REPO}/actions"
echo "2. Check Launchpad builds: https://launchpad.net/~ashishtyagi10/+archive/ubuntu/cortex"
echo "3. Once built, test with:"
echo "   sudo add-apt-repository ppa:ashishtyagi10/cortex"
echo "   sudo apt update"
echo "   sudo apt install cortex"