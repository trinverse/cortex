#!/bin/bash

# Generate PNG icons from SVG sources
# Requires: inkscape or rsvg-convert

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Cortex Icon Generator${NC}"
echo "========================"

# Check for required tools
if command -v inkscape &> /dev/null; then
    CONVERTER="inkscape"
    echo -e "${GREEN}✓${NC} Using Inkscape for conversion"
elif command -v rsvg-convert &> /dev/null; then
    CONVERTER="rsvg-convert"
    echo -e "${GREEN}✓${NC} Using rsvg-convert for conversion"
else
    echo -e "${RED}✗${NC} Neither inkscape nor rsvg-convert found!"
    echo "Please install one of them:"
    echo "  Ubuntu/Debian: sudo apt-get install inkscape"
    echo "  or: sudo apt-get install librsvg2-bin"
    echo "  macOS: brew install inkscape"
    echo "  or: brew install librsvg"
    exit 1
fi

# Create PNG directory
mkdir -p png

# Function to convert SVG to PNG
convert_svg_to_png() {
    local svg_file=$1
    local size=$2
    local output_file=$3
    
    if [ "$CONVERTER" = "inkscape" ]; then
        inkscape "$svg_file" --export-type=png --export-filename="$output_file" \
                 --export-width=$size --export-height=$size 2>/dev/null
    else
        rsvg-convert -w $size -h $size "$svg_file" -o "$output_file"
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} Generated $output_file"
    else
        echo -e "  ${RED}✗${NC} Failed to generate $output_file"
    fi
}

# Generate main icon in various sizes
echo -e "\n${YELLOW}Generating main icons...${NC}"
for size in 16 24 32 48 64 96 128 256 512 1024; do
    convert_svg_to_png "svg/cortex-icon.svg" $size "png/cortex-${size}.png"
done

# Generate simple icon versions
echo -e "\n${YELLOW}Generating simple icons...${NC}"
for size in 16 24 32 48 64 128; do
    convert_svg_to_png "svg/cortex-icon-simple.svg" $size "png/cortex-simple-${size}.png"
done

# Generate symbolic icons
echo -e "\n${YELLOW}Generating symbolic icons...${NC}"
for size in 16 24 32 48; do
    convert_svg_to_png "svg/cortex-icon-symbolic.svg" $size "png/cortex-symbolic-${size}.png"
done

# Generate favicon sizes
echo -e "\n${YELLOW}Generating favicon sizes...${NC}"
for size in 16 32 48 64; do
    convert_svg_to_png "svg/cortex-favicon.svg" $size "png/favicon-${size}.png"
done

# Generate Apple Touch Icon
echo -e "\n${YELLOW}Generating Apple Touch Icon...${NC}"
convert_svg_to_png "svg/cortex-icon-simple.svg" 180 "png/apple-touch-icon.png"

# Generate Windows ICO file (requires ImageMagick)
if command -v convert &> /dev/null; then
    echo -e "\n${YELLOW}Generating Windows ICO...${NC}"
    convert png/cortex-16.png png/cortex-32.png png/cortex-48.png \
            png/cortex-64.png png/cortex-128.png png/cortex-256.png \
            ico/cortex.ico
    echo -e "  ${GREEN}✓${NC} Generated ico/cortex.ico"
else
    echo -e "\n${YELLOW}⚠${NC} ImageMagick not found, skipping ICO generation"
    echo "  Install with: sudo apt-get install imagemagick"
fi

# Generate macOS ICNS file (requires iconutil on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "\n${YELLOW}Generating macOS ICNS...${NC}"
    
    # Create iconset directory
    mkdir -p cortex.iconset
    
    # Copy and rename files according to Apple's requirements
    cp png/cortex-16.png cortex.iconset/icon_16x16.png
    cp png/cortex-32.png cortex.iconset/icon_16x16@2x.png
    cp png/cortex-32.png cortex.iconset/icon_32x32.png
    cp png/cortex-64.png cortex.iconset/icon_32x32@2x.png
    cp png/cortex-128.png cortex.iconset/icon_128x128.png
    cp png/cortex-256.png cortex.iconset/icon_128x128@2x.png
    cp png/cortex-256.png cortex.iconset/icon_256x256.png
    cp png/cortex-512.png cortex.iconset/icon_256x256@2x.png
    cp png/cortex-512.png cortex.iconset/icon_512x512.png
    cp png/cortex-1024.png cortex.iconset/icon_512x512@2x.png
    
    # Generate ICNS
    iconutil -c icns cortex.iconset -o cortex.icns
    
    # Clean up
    rm -rf cortex.iconset
    
    echo -e "  ${GREEN}✓${NC} Generated cortex.icns"
fi

# Create Linux desktop file
echo -e "\n${YELLOW}Creating Linux desktop file...${NC}"
cat > ../../cortex.desktop << EOF
[Desktop Entry]
Name=Cortex
Comment=Modern Terminal File Manager
Exec=cortex
Icon=cortex
Terminal=true
Type=Application
Categories=System;FileTools;FileManager;
Keywords=files;folders;directory;manager;
StartupNotify=false
EOF
echo -e "  ${GREEN}✓${NC} Created cortex.desktop"

# Summary
echo -e "\n${GREEN}Icon generation complete!${NC}"
echo "========================"
echo "Generated files:"
echo "  • PNG icons in various sizes (png/)"
echo "  • Desktop file (cortex.desktop)"

if [ -f "ico/cortex.ico" ]; then
    echo "  • Windows ICO file (ico/cortex.ico)"
fi

if [ -f "cortex.icns" ]; then
    echo "  • macOS ICNS file (cortex.icns)"
fi

echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Review generated icons"
echo "2. Test on different platforms"
echo "3. Include in build process"