# Command Palette Feature

## Overview

The command palette provides a **type-ahead interface** for all commands in Cortex. When you type `/`, a filterable list of all available commands appears instantly.

## How It Works

### Activation
- Type `/` on an empty command line → Command palette opens
- Shows all available commands organized by category
- Real-time filtering as you type

### Features
- **Type-ahead filtering**: Commands filter as you type
- **Category grouping**: Commands organized by type (Files, Navigation, View, etc.)
- **Keyboard shortcuts displayed**: Shows associated shortcuts for each command
- **Smart matching**: Searches command names, descriptions, and categories
- **Autocomplete**: Tab key completes the selected command

## Usage

### Basic Flow
1. Press `/` to open command palette
2. See all available commands instantly
3. Start typing to filter (e.g., `/co` shows copy, `/vi` shows view)
4. Use ↑↓ arrows to navigate suggestions
5. Press Enter to execute selected command
6. Press Tab to autocomplete
7. Press ESC to cancel

### Available Commands

#### System Commands
- `/exit` - Exit Cortex (Ctrl+Q)
- `/quit` - Quit Cortex  
- `/reload` - Reload file panels (Ctrl+R)
- `/help` - Show help dialog (F1)

#### File Operations
- `/copy` - Copy selected files (F5)
- `/move` - Move selected files (F6)
- `/delete` - Delete selected files (F8)
- `/mkdir` - Create new directory (F7)
- `/rename` - Rename current file

#### View Commands
- `/view` - View current file (F3)
- `/edit` - Edit current file (F4)
- `/filter` - Quick filter panel (Ctrl+F)
- `/hidden` - Toggle hidden files (Ctrl+H)

#### Navigation
- `/home` - Go to home directory
- `/root` - Go to root directory
- `/cd <path>` - Change to specific directory

#### Search
- `/find` - Find files by name (Alt+F7)
- `/filter` - Quick filter current panel (Ctrl+F)

## Visual Indicators

```
┌─ Command Palette ──────────────────────────┐
│ /cop                                       │ <- Input area with cursor
├────────────────────────────────────────────┤
│  Files                                     │ <- Category header
│    /copy     Copy selected files    [F5]  │ <- Matching command
│                                            │
│  Navigation                                │
│    /cd       Change directory              │
├────────────────────────────────────────────┤
│ ↑↓: Navigate | Enter: Execute | Tab: Auto  │ <- Help line
└────────────────────────────────────────────┘
```

## Smart Features

### Filtering Logic
- Matches are case-insensitive
- Searches in:
  - Command names (without the `/`)
  - Command descriptions
  - Category names
- Prioritizes commands that start with the search term

### Relevance Sorting
1. Commands starting with search term appear first
2. Then other matches sorted by category
3. Within categories, sorted alphabetically

## Examples

### Quick Copy
1. Type `/`
2. Type `co` → `/copy` is highlighted
3. Press Enter → Copy dialog opens

### Navigate Home
1. Type `/`
2. Type `ho` → `/home` is highlighted
3. Press Enter → Navigate to home directory

### Filter Files
1. Type `/`
2. Type `fil` → `/filter` is highlighted
3. Press Enter → Filter dialog opens

## Implementation Details

### Files Added
- `cortex-cli/src/command_palette.rs` - Command registry and logic
- `cortex-tui/src/command_palette_dialog.rs` - UI component

### Key Components
- **CommandPaletteDialog**: Main dialog component
- **CommandInfo**: Structure for command metadata
- Real-time filtering in `filter_commands()`
- Category-based grouping for display

### Integration
- Triggers on `/` key when command line is empty
- Executes through existing `handle_special_command()`
- Seamlessly integrates with existing command system

## Benefits

1. **Discoverability**: Users can see all available commands
2. **Speed**: Type-ahead makes command execution faster
3. **Learning**: Shows keyboard shortcuts alongside commands
4. **Consistency**: Unified interface for all commands
5. **Efficiency**: No need to memorize command names

## Future Enhancements

- Command usage history/frecency
- Fuzzy matching algorithm
- Command aliases
- Custom command definitions
- Command preview/help
- Multi-step command wizards