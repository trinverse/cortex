# 📝 Command Mode Guide for Cortex

## How to Enter Command Mode

You have **two options** to activate command mode:

### Option 1: Press `Ctrl+O` (Control + O)
- Hold Control key
- Press O key
- The command area border will turn **cyan**
- You'll see "Command Mode (ESC to exit)" at the top

### Option 2: Press `:` (colon)
- Just press the colon key `:`
- Same as vim/vi editors
- The command area will activate

## Visual Indicators

When **NOT** in command mode:
- Border is **gray**
- Shows: "Press Ctrl+O or : to enter command mode"
- Prompt shows: "$ (Ctrl+O to activate)"

When **IN** command mode:
- Border turns **cyan** (blue)
- Shows: "Command Mode (ESC to exit)"
- Cursor appears after `$`
- You can type commands!

## Using Command Mode

1. **Enter command mode**: Press `Ctrl+O` or `:`
2. **Type your command**: 
   - `ls` - list files
   - `pwd` - show current directory
   - `cd /path` - change directory
   - `vim %f` - edit current file
   - Any shell command!
3. **Execute**: Press `Enter`
4. **Exit command mode**: Press `ESC`

## Special Variables

- `%f` - Current file name
- `%F` - All marked files
- `%d` - Current directory
- `%D` - Other panel directory

## Examples

```bash
# After pressing Ctrl+O:
ls -la           # List all files
cat %f           # View current file
cp %F /tmp       # Copy marked files to /tmp
grep "text" %f   # Search in current file
```

## Troubleshooting

### Can't enter command mode?
- Make sure no dialog is open (press ESC first)
- Try both `Ctrl+O` and `:` 
- Look for the cyan border when active

### Command not working?
- Make sure you're in command mode (cyan border)
- Press Enter to execute
- Check for typos

### Exit command mode
- Press `ESC` key
- The border returns to gray

## Quick Test

1. Run Cortex: `./target/debug/cortex`
2. Press `Ctrl+O` (you should see cyan border)
3. Type: `echo "Hello from command mode!"`
4. Press `Enter`
5. Press `ESC` to exit command mode