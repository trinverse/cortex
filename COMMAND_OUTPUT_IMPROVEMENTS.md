# Command Output Panel Improvements

## Problem
The Command Output panel couldn't be closed with the Esc key, making it unintuitive to dismiss. The 'O' key toggle was the only way to close it, which wasn't discoverable.

## Solution Implemented

### 1. **Esc Key Support**
- The Esc key now intelligently closes the Command Output panel when it's visible
- If the panel is not visible, Esc maintains its original behavior (clearing command line)
- Priority: Panel closing takes precedence over command line clearing

### 2. **Visual Feedback**
- Added status messages when opening/closing the panel
- "Command output panel opened" when pressing 'O' to open
- "Command output panel closed" when pressing 'O' or Esc to close

### 3. **Updated Help Text**
- Panel title now shows: `[Esc to close, O to toggle]`
- Makes the closing mechanism discoverable to users

### 4. **New API Method**
- Added `hide_command_output()` method to AppState for explicit hiding
- Maintains separation between toggle and hide operations

## Key Bindings

| Key | Action | Context |
|-----|--------|---------|
| **Esc** | Close command output panel | When panel is visible |
| **Esc** | Clear command line | When panel is hidden |
| **O** | Toggle command output panel | When command line is empty |

## Code Changes

### Files Modified:
1. **cortex-cli/src/main.rs**
   - Updated Esc key handler (line 864-877)
   - Enhanced 'O' key handler with feedback (line 963-973)

2. **cortex-core/src/state.rs**
   - Added `hide_command_output()` method (line 606-608)

3. **cortex-tui/src/ui.rs**
   - Updated panel title with help text (line 590-593)

## User Experience Benefits
- **Intuitive**: Esc is the universal "close/cancel" key
- **Discoverable**: Help text shows available actions
- **Feedback**: Status messages confirm actions
- **Consistent**: Follows standard UI patterns