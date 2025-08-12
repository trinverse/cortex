# Refactoring Plan for Cortex

## Overview
Refactoring main.rs (2931 lines) and vfs.rs (613 lines) into smaller, focused modules following clean code principles.

## 1. Main.rs Refactoring

### Current Structure Analysis
The main.rs file contains:
- Application lifecycle management
- Event handling (keyboard, mouse)
- Dialog management
- Command processing
- File operations
- UI updates
- Plugin management
- Configuration handling
- Connection management (SSH/FTP)

### Proposed Module Structure

```
cortex-cli/src/
├── main.rs (< 200 lines - just entry point and App struct)
├── app/
│   ├── mod.rs
│   ├── lifecycle.rs    (initialization, cleanup)
│   └── state.rs        (app state management)
├── handlers/
│   ├── mod.rs
│   ├── keyboard.rs     (keyboard event handling)
│   ├── mouse.rs        (mouse event handling)
│   ├── dialog.rs       (dialog input handling)
│   └── events.rs       (event orchestration)
├── commands/
│   ├── mod.rs
│   ├── special.rs      (/ commands)
│   ├── navigation.rs   (cd, navigation)
│   └── file_ops.rs     (copy, move, delete)
├── dialogs/
│   ├── mod.rs
│   └── handlers.rs     (dialog-specific logic)
├── connections/
│   ├── mod.rs
│   ├── ssh.rs          (SSH/SFTP connection)
│   └── ftp.rs          (FTP connection)
└── ui/
    ├── mod.rs
    └── updates.rs      (UI refresh logic)
```

### Key Extractions from main.rs:

1. **app/lifecycle.rs** (~300 lines)
   - `App::new()` initialization
   - `cleanup_and_exit()`
   - Terminal setup/teardown
   - Plugin loading

2. **handlers/keyboard.rs** (~600 lines)
   - `handle_input()` method
   - Function key handlers (F1-F10)
   - Control key combinations
   - Navigation keys

3. **handlers/dialog.rs** (~400 lines)
   - `handle_dialog_input()` method
   - Dialog-specific key handling

4. **commands/special.rs** (~300 lines)
   - `handle_special_command()` method
   - All /command implementations

5. **connections/ssh.rs** (~100 lines)
   - `connect_sftp()` method
   - SSH credential handling

6. **handlers/mouse.rs** (~200 lines)
   - `handle_mouse_event()` method
   - Context menu handling

## 2. VFS.rs Refactoring

### Current Structure Analysis
The vfs.rs file contains:
- VFS trait and core types
- Local filesystem provider
- Archive provider
- SSH/SFTP provider
- FTP provider
- Connection management

### Proposed Module Structure

```
cortex-core/src/vfs/
├── mod.rs              (< 100 lines - core traits and types)
├── types.rs            (VfsPath, VfsEntry, etc.)
├── providers/
│   ├── mod.rs
│   ├── local.rs        (LocalFileSystemProvider)
│   ├── archive.rs      (ArchiveProvider)
│   ├── ssh/
│   │   ├── mod.rs
│   │   ├── connection.rs (SshConnectionManager)
│   │   └── sftp.rs     (SftpProvider)
│   └── ftp.rs          (FtpProvider)
└── builder.rs          (VirtualFileSystemBuilder)
```

### Key Extractions from vfs.rs:

1. **vfs/types.rs** (~80 lines)
   - `VfsPath` enum
   - `VfsEntry` struct
   - `VfsEntryType` enum
   - `RemoteCredentials` struct

2. **vfs/providers/local.rs** (~130 lines)
   - `LocalFileSystemProvider` implementation

3. **vfs/providers/ssh/connection.rs** (~50 lines)
   - `SshConnectionManager` struct

4. **vfs/providers/ssh/sftp.rs** (~180 lines)
   - `SftpProvider` implementation

5. **vfs/providers/archive.rs** (~50 lines)
   - `ArchiveProvider` implementation

6. **vfs/providers/ftp.rs** (~50 lines)
   - `FtpProvider` implementation

## 3. Implementation Order

1. **Phase 1**: Create directory structure
2. **Phase 2**: Extract VFS modules (simpler, self-contained)
3. **Phase 3**: Extract main.rs handlers
4. **Phase 4**: Extract command processing
5. **Phase 5**: Clean up and optimize imports

## 4. Benefits

- **Better maintainability**: Each file has a single responsibility
- **Improved testability**: Smaller units are easier to test
- **Faster compilation**: Only changed modules recompile
- **Clearer dependencies**: Explicit imports show relationships
- **Easier collaboration**: Less merge conflicts
- **Better code navigation**: Find code by its purpose

## 5. Verification Steps

After each phase:
1. Run `cargo build`
2. Run `cargo clippy -- -W clippy::all`
3. Run `cargo test`
4. Test application functionality manually