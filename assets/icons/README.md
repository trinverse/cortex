# Cortex Icons

This directory contains all icon assets for the Cortex File Manager application.

## Icon Variants

### Main Icons

1. **cortex-icon.svg** - Primary application icon with detailed design
   - Features dual-pane representation
   - Gradient background with modern styling
   - Best for sizes 64x64 and larger

2. **cortex-icon-simple.svg** - Simplified version for better scalability
   - Cleaner geometry for small sizes
   - Maintains dual-pane concept
   - Best for sizes 32x32 to 128x128

3. **cortex-icon-alt.svg** - Alternative modern design
   - Minimalist geometric approach
   - Different gradient palette
   - Contemporary flat design

4. **cortex-icon-symbolic.svg** - Monochrome symbolic icon
   - For system trays and symbolic contexts
   - Uses currentColor for theming
   - Optimized for 16x16 and 24x24

5. **cortex-favicon.svg** - Optimized favicon
   - Designed for web and small displays
   - High contrast at small sizes
   - 32x32 optimized

6. **cortex-logo.svg** - Full logo with text
   - Horizontal layout with "CORTEX" text
   - For marketing and documentation
   - Not for application icons

## Color Palette

### Primary Gradient
- Start: `#667eea` (Indigo)
- End: `#764ba2` (Purple)

### Accent Gradient
- Start: `#f093fb` (Pink)
- End: `#f5576c` (Coral)

### Alternative Palette
- Primary: `#6366f1` (Indigo)
- Secondary: `#8b5cf6` (Purple)
- Accent: `#ec4899` (Pink)

## Usage

### Generating PNG Icons

Run the generation script to create PNG versions:

```bash
cd assets/icons
./generate-icons.sh
```

This will create:
- PNG icons in various sizes (16px to 1024px)
- Windows ICO file (if ImageMagick installed)
- macOS ICNS file (if on macOS)
- Linux desktop file

### Platform Integration

#### Linux
```bash
# Install icon to system
sudo cp png/cortex-*.png /usr/share/icons/hicolor/{size}x{size}/apps/
sudo cp svg/cortex-icon.svg /usr/share/icons/hicolor/scalable/apps/cortex.svg
sudo cp ../cortex.desktop /usr/share/applications/

# Update icon cache
sudo gtk-update-icon-cache /usr/share/icons/hicolor/
```

#### Windows
- Use `cortex.ico` for executable icon
- Set in `Cargo.toml` with `winres` crate
- Include in WiX installer configuration

#### macOS
- Use `cortex.icns` for app bundle
- Place in `Cortex.app/Contents/Resources/`
- Reference in `Info.plist`

### Web Usage

```html
<!-- Favicon -->
<link rel="icon" type="image/svg+xml" href="/assets/icons/svg/cortex-favicon.svg">
<link rel="alternate icon" href="/assets/icons/png/favicon-32.png">

<!-- Apple Touch Icon -->
<link rel="apple-touch-icon" href="/assets/icons/png/apple-touch-icon.png">

<!-- Open Graph -->
<meta property="og:image" content="/assets/icons/png/cortex-512.png">
```

## Icon Guidelines

### When to Use Each Variant

- **Application Icon**: Use `cortex-icon.svg` for sizes ≥ 64px
- **Small Icons**: Use `cortex-icon-simple.svg` for 32-48px
- **System Tray**: Use `cortex-icon-symbolic.svg`
- **Documentation**: Use `cortex-logo.svg` with text
- **Web Favicon**: Use `cortex-favicon.svg`

### Maintaining Consistency

1. Always maintain the dual-pane concept
2. Keep gradient direction consistent (top-left to bottom-right)
3. Ensure sufficient contrast for accessibility
4. Test icons at target sizes before deployment

## Design Rationale

The Cortex icon design represents:
- **Dual-pane layout**: Core feature of the file manager
- **Modern gradients**: Contemporary, professional appearance
- **Geometric shapes**: Technical, organized nature
- **Purple/Indigo palette**: Unique brand identity

## Tools Used

- **Design**: Hand-coded SVG for precision
- **Optimization**: SVGO for file size reduction
- **Conversion**: Inkscape/rsvg-convert for PNG generation
- **Platform formats**: iconutil (macOS), ImageMagick (Windows)

## License

These icons are part of the Cortex project and are licensed under the same terms as the main application (MIT License).

## Credits

Icons designed for the Cortex File Manager project.
Created: January 2025