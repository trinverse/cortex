use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Keyboard shortcut definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub code: String,
    pub modifiers: Vec<String>,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        let code_str = match code {
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            _ => "Unknown".to_string(),
        };

        let mut modifier_vec = Vec::new();
        if modifiers.contains(KeyModifiers::CONTROL) {
            modifier_vec.push("Ctrl".to_string());
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            modifier_vec.push("Shift".to_string());
        }
        if modifiers.contains(KeyModifiers::ALT) {
            modifier_vec.push("Alt".to_string());
        }

        Self {
            code: code_str,
            modifiers: modifier_vec,
        }
    }

    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        let binding = Self::new(code, modifiers);
        self.code == binding.code && self.modifiers == binding.modifiers
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.code)
        } else {
            write!(f, "{}+{}", self.modifiers.join("+"), self.code)
        }
    }
}

/// Action that can be triggered by a keyboard shortcut
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigatePageUp,
    NavigatePageDown,
    NavigateHome,
    NavigateEnd,
    NavigateToParent,
    NavigateInto,
    NavigateBack,
    NavigateForward,
    GoToRootDir,
    GoToDeepestDir,

    // File Operations
    Copy,
    CopyAs,
    Move,
    Delete,
    DeleteToTrash,
    RestoreFromTrash,
    Rename,
    CreateFile,
    CreateDirectory,
    CopyToClipboard,
    PasteFromClipboard,
    ForceDelete,
    NewFile,
    NewFolder,

    // Selection
    SelectItem,
    SelectAll,
    SelectNone,
    InvertSelection,
    SelectPattern,
    DeselectPattern,

    // View
    ViewFile,
    EditFile,
    QuickView,
    ToggleHidden,
    ToggleDetails,
    Refresh,
    ToggleTreeView,
    BriefView,
    FullView,
    WideView,

    // Sorting
    SortByName,
    SortByExtension,
    SortByDate,
    SortBySize,
    ReverseSort,

    // Search
    Search,
    SearchNext,
    SearchPrevious,
    QuickFilter,
    FindInFiles,
    ClearFilter,
    GoToLine,

    // Panels
    SwitchPanel,
    SwapPanels,
    SyncPanels,
    EqualPanels,
    FocusLeftPanel,
    FocusRightPanel,
    HidePanels,
    PanelMenu,
    TreePanel,
    ChangeDriveLeft,
    ChangeDriveRight,

    // Commands
    CommandPalette,
    CommandLine,
    ExecuteCommand,
    ShellCommand,
    RunInTerminal,
    Autocomplete,

    // System
    Help,
    Settings,
    Quit,
    QuickExit,
    ShowShortcuts,
    ToggleFullscreen,
    OpenSystemTerminal,
    ContextHelp,
    About,

    // Bookmarks
    AddBookmark,
    ShowBookmarks,
    GoToBookmark(u8), // 1-9
    ManageBookmarks,

    // Quick Directories
    GoToHome,
    GoToRoot,
    GoToDesktop,
    GoToDocuments,
    GoToDownloads,
    GoToQuickDir(u8), // 1-9
    SetQuickDir(u8), // 1-9

    // History
    ShowHistory,
    HistoryBack,
    HistoryForward,
    ShowHistoryList,

    // Clipboard
    Cut,
    Paste,

    // Advanced
    OpenTerminal,
    OpenWith,
    Properties,
    CompareFiles,
    SyncDirectories,
    FindDuplicates,
    EnterArchive,
    ExtractArchive,
    CreateArchive,
    SftpConnect,
    FtpConnect,
    Disconnect,
    CompareDirs,
    CalculateSize,
    MultiRename,

    // Macros
    StartMacroRecord,
    PlayMacro,
    ManageMacros,

    // Vim Mode Actions
    VimEnterNormal,
    VimEnterInsert,
    VimEnterVisual,
    VimEnterCommand,
    VimYank,
    VimPaste,
    VimDelete,
    VimUndo,
    VimRedo,
    VimSearch,
    VimGoToTop,
    VimGoToBottom,
    VimGoToLine,
    VimMark,
    VimGoToMark,
}

/// Vim mode for modal editing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

/// Keyboard shortcut manager
pub struct ShortcutManager {
    shortcuts: HashMap<KeyBinding, Action>,
    vim_mode: Option<VimMode>,
    custom_shortcuts: HashMap<KeyBinding, Action>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();

        // Function Keys (F1-F10)
        shortcuts.insert(KeyBinding { code: "F1".to_string(), modifiers: vec![] }, Action::Help);
        shortcuts.insert(KeyBinding { code: "F2".to_string(), modifiers: vec![] }, Action::Rename);
        shortcuts.insert(KeyBinding { code: "F3".to_string(), modifiers: vec![] }, Action::ViewFile);
        shortcuts.insert(KeyBinding { code: "F4".to_string(), modifiers: vec![] }, Action::EditFile);
        shortcuts.insert(KeyBinding { code: "F5".to_string(), modifiers: vec![] }, Action::Copy);
        shortcuts.insert(KeyBinding { code: "F6".to_string(), modifiers: vec![] }, Action::Move);
        shortcuts.insert(KeyBinding { code: "F7".to_string(), modifiers: vec![] }, Action::CreateDirectory);
        shortcuts.insert(KeyBinding { code: "F8".to_string(), modifiers: vec![] }, Action::Delete);
        shortcuts.insert(KeyBinding { code: "F9".to_string(), modifiers: vec![] }, Action::Settings);
        shortcuts.insert(KeyBinding { code: "F10".to_string(), modifiers: vec![] }, Action::Quit);
        shortcuts.insert(KeyBinding { code: "F11".to_string(), modifiers: vec![] }, Action::ToggleFullscreen);
        shortcuts.insert(KeyBinding { code: "F12".to_string(), modifiers: vec![] }, Action::OpenSystemTerminal);

        // Extended Function Keys
        shortcuts.insert(KeyBinding { code: "F1".to_string(), modifiers: vec!["Shift".to_string()] }, Action::ContextHelp);
        shortcuts.insert(KeyBinding { code: "F1".to_string(), modifiers: vec!["Alt".to_string()] }, Action::ChangeDriveLeft);
        shortcuts.insert(KeyBinding { code: "F2".to_string(), modifiers: vec!["Alt".to_string()] }, Action::ChangeDriveRight);
        shortcuts.insert(KeyBinding { code: "F3".to_string(), modifiers: vec!["Shift".to_string()] }, Action::SearchPrevious);
        shortcuts.insert(KeyBinding { code: "F4".to_string(), modifiers: vec!["Shift".to_string()] }, Action::NewFile);
        shortcuts.insert(KeyBinding { code: "F5".to_string(), modifiers: vec!["Shift".to_string()] }, Action::CopyAs);
        shortcuts.insert(KeyBinding { code: "F6".to_string(), modifiers: vec!["Shift".to_string()] }, Action::Rename);
        shortcuts.insert(KeyBinding { code: "F7".to_string(), modifiers: vec!["Alt".to_string()] }, Action::FindInFiles);
        shortcuts.insert(KeyBinding { code: "F8".to_string(), modifiers: vec!["Alt".to_string()] }, Action::ShowHistory);

        // Navigation
        shortcuts.insert(KeyBinding { code: "Up".to_string(), modifiers: vec![] }, Action::NavigateUp);
        shortcuts.insert(KeyBinding { code: "Down".to_string(), modifiers: vec![] }, Action::NavigateDown);
        shortcuts.insert(KeyBinding { code: "Left".to_string(), modifiers: vec![] }, Action::NavigateToParent);
        shortcuts.insert(KeyBinding { code: "Right".to_string(), modifiers: vec![] }, Action::NavigateInto);
        shortcuts.insert(KeyBinding { code: "Enter".to_string(), modifiers: vec![] }, Action::NavigateInto);
        shortcuts.insert(KeyBinding { code: "Backspace".to_string(), modifiers: vec![] }, Action::NavigateToParent);
        shortcuts.insert(KeyBinding { code: "Tab".to_string(), modifiers: vec![] }, Action::SwitchPanel);
        shortcuts.insert(KeyBinding { code: "PageUp".to_string(), modifiers: vec![] }, Action::NavigatePageUp);
        shortcuts.insert(KeyBinding { code: "PageDown".to_string(), modifiers: vec![] }, Action::NavigatePageDown);
        shortcuts.insert(KeyBinding { code: "Home".to_string(), modifiers: vec![] }, Action::NavigateHome);
        shortcuts.insert(KeyBinding { code: "End".to_string(), modifiers: vec![] }, Action::NavigateEnd);
        shortcuts.insert(KeyBinding { code: "PageUp".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToRootDir);
        shortcuts.insert(KeyBinding { code: "PageDown".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToDeepestDir);
        shortcuts.insert(KeyBinding { code: "Left".to_string(), modifiers: vec!["Alt".to_string()] }, Action::HistoryBack);
        shortcuts.insert(KeyBinding { code: "Right".to_string(), modifiers: vec!["Alt".to_string()] }, Action::HistoryForward);
        shortcuts.insert(KeyBinding { code: "Down".to_string(), modifiers: vec!["Alt".to_string()] }, Action::ShowHistoryList);

        // File Selection
        shortcuts.insert(KeyBinding { code: "Space".to_string(), modifiers: vec![] }, Action::SelectItem);
        shortcuts.insert(KeyBinding { code: "Insert".to_string(), modifiers: vec![] }, Action::SelectItem);
        shortcuts.insert(KeyBinding { code: "a".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SelectAll);
        shortcuts.insert(KeyBinding { code: "u".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SelectNone);
        shortcuts.insert(KeyBinding { code: "+".to_string(), modifiers: vec![] }, Action::SelectPattern);
        shortcuts.insert(KeyBinding { code: "-".to_string(), modifiers: vec![] }, Action::DeselectPattern);
        shortcuts.insert(KeyBinding { code: "*".to_string(), modifiers: vec![] }, Action::InvertSelection);

        // File Operations
        shortcuts.insert(KeyBinding { code: "c".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::CopyToClipboard);
        shortcuts.insert(KeyBinding { code: "x".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::Cut);
        shortcuts.insert(KeyBinding { code: "v".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::PasteFromClipboard);
        shortcuts.insert(KeyBinding { code: "Delete".to_string(), modifiers: vec![] }, Action::Delete);
        shortcuts.insert(KeyBinding { code: "Delete".to_string(), modifiers: vec!["Shift".to_string()] }, Action::ForceDelete);
        shortcuts.insert(KeyBinding { code: "n".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::NewFile);
        shortcuts.insert(KeyBinding { code: "n".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::NewFolder);

        // Search & Filter
        shortcuts.insert(KeyBinding { code: "f".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::QuickFilter);
        shortcuts.insert(KeyBinding { code: "f".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::FindInFiles);
        shortcuts.insert(KeyBinding { code: "/".to_string(), modifiers: vec![] }, Action::QuickFilter);
        shortcuts.insert(KeyBinding { code: "l".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ClearFilter);
        shortcuts.insert(KeyBinding { code: "g".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToLine);
        // Removed: shortcuts.insert(KeyBinding { code: "F3".to_string(), modifiers: vec![] }, Action::SearchNext); // Conflict with Action::ViewFile
        // Removed: shortcuts.insert(KeyBinding { code: "n".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SearchNext); // Conflict with Action::NewFile
        shortcuts.insert(KeyBinding { code: "n".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::SearchPrevious);

        // View Options
        shortcuts.insert(KeyBinding { code: "h".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ToggleHidden);
        shortcuts.insert(KeyBinding { code: "d".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ToggleDetails);
        shortcuts.insert(KeyBinding { code: "t".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ToggleTreeView);
        shortcuts.insert(KeyBinding { code: "1".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::BriefView);
        shortcuts.insert(KeyBinding { code: "2".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::FullView);
        shortcuts.insert(KeyBinding { code: "3".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::WideView);
        shortcuts.insert(KeyBinding { code: "F3".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SortByName);
        shortcuts.insert(KeyBinding { code: "F4".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SortByExtension);
        shortcuts.insert(KeyBinding { code: "F5".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SortByDate);
        shortcuts.insert(KeyBinding { code: "F6".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SortBySize);
        shortcuts.insert(KeyBinding { code: "r".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ReverseSort);

        // Panel Operations
        shortcuts.insert(KeyBinding { code: "u".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SwapPanels);
        shortcuts.insert(KeyBinding { code: "s".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::SyncPanels);
        shortcuts.insert(KeyBinding { code: "=".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::EqualPanels);
        shortcuts.insert(KeyBinding { code: "l".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::FocusLeftPanel);
        shortcuts.insert(KeyBinding { code: "r".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::FocusRightPanel);
        shortcuts.insert(KeyBinding { code: "o".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::HidePanels);
        shortcuts.insert(KeyBinding { code: "p".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::PanelMenu);
        shortcuts.insert(KeyBinding { code: "F10".to_string(), modifiers: vec!["Alt".to_string()] }, Action::TreePanel);

        // Quick Access
        for i in 1..=9 {
            shortcuts.insert(KeyBinding { code: i.to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToQuickDir(i));
            shortcuts.insert(KeyBinding { code: i.to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::SetQuickDir(i));
        }
        shortcuts.insert(KeyBinding { code: "0".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToHome);
        shortcuts.insert(KeyBinding { code: "/".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::GoToRoot);

        // Bookmarks
        shortcuts.insert(KeyBinding { code: "d".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::AddBookmark);
        shortcuts.insert(KeyBinding { code: "b".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ShowBookmarks);
        for i in 1..=9 {
            shortcuts.insert(KeyBinding { code: i.to_string(), modifiers: vec!["Alt".to_string()] }, Action::GoToBookmark(i));
        }
        shortcuts.insert(KeyBinding { code: "b".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::ManageBookmarks);

        // Command Line
        shortcuts.insert(KeyBinding { code: ":".to_string(), modifiers: vec![] }, Action::CommandLine);
        shortcuts.insert(KeyBinding { code: "!".to_string(), modifiers: vec![] }, Action::ShellCommand);
        shortcuts.insert(KeyBinding { code: "Enter".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::RunInTerminal);
        shortcuts.insert(KeyBinding { code: "Space".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::Autocomplete);

        // Advanced Features
        shortcuts.insert(KeyBinding { code: "PageDown".to_string(), modifiers: vec!["Alt".to_string()] }, Action::EnterArchive);
        shortcuts.insert(KeyBinding { code: "F6".to_string(), modifiers: vec!["Alt".to_string()] }, Action::ExtractArchive);
        shortcuts.insert(KeyBinding { code: "F5".to_string(), modifiers: vec!["Alt".to_string()] }, Action::CreateArchive);
        shortcuts.insert(KeyBinding { code: "s".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::SftpConnect);
        shortcuts.insert(KeyBinding { code: "f".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::FtpConnect);
        shortcuts.insert(KeyBinding { code: "d".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::Disconnect);
        shortcuts.insert(KeyBinding { code: "k".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::CompareDirs);
        shortcuts.insert(KeyBinding { code: "c".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::CalculateSize);
        shortcuts.insert(KeyBinding { code: "m".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::MultiRename);
        shortcuts.insert(KeyBinding { code: "Enter".to_string(), modifiers: vec!["Alt".to_string()] }, Action::Properties);

        // Special Keys
        shortcuts.insert(KeyBinding { code: "q".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::QuickExit);
        shortcuts.insert(KeyBinding { code: "?".to_string(), modifiers: vec!["Ctrl".to_string()] }, Action::ShowShortcuts);
        shortcuts.insert(KeyBinding { code: "F1".to_string(), modifiers: vec!["Alt".to_string()] }, Action::About);

        // Macros
        shortcuts.insert(KeyBinding { code: "r".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::StartMacroRecord);
        shortcuts.insert(KeyBinding { code: "p".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::PlayMacro);
        shortcuts.insert(KeyBinding { code: "m".to_string(), modifiers: vec!["Ctrl".to_string(), "Shift".to_string()] }, Action::ManageMacros);

        Self {
            shortcuts,
            vim_mode: None,
            custom_shortcuts: HashMap::new(),
        }
    }

    /// Get action for a key combination
    pub fn get_action(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        let binding = KeyBinding::new(code, modifiers);

        // Check custom shortcuts first
        if let Some(action) = self.custom_shortcuts.get(&binding) {
            return Some(action.clone());
        }

        // Check vim mode specific bindings
        if let Some(vim_mode) = self.vim_mode {
            if let Some(action) = self.get_vim_action(vim_mode, code, modifiers) {
                return Some(action);
            }
        }

        // Check standard shortcuts
        self.shortcuts.get(&binding).cloned()
    }

    /// Get vim mode specific action
    fn get_vim_action(
        &self,
        mode: VimMode,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<Action> {
        match mode {
            VimMode::Normal => match (code, modifiers) {
                (KeyCode::Char('i'), KeyModifiers::NONE) => Some(Action::VimEnterInsert),
                (KeyCode::Char('v'), KeyModifiers::NONE) => Some(Action::VimEnterVisual),
                (KeyCode::Char(':'), KeyModifiers::NONE) => Some(Action::VimEnterCommand),
                (KeyCode::Char('h'), KeyModifiers::NONE) => Some(Action::NavigateLeft),
                (KeyCode::Char('j'), KeyModifiers::NONE) => Some(Action::NavigateDown),
                (KeyCode::Char('k'), KeyModifiers::NONE) => Some(Action::NavigateUp),
                (KeyCode::Char('l'), KeyModifiers::NONE) => Some(Action::NavigateRight),
                (KeyCode::Char('g'), KeyModifiers::NONE) => Some(Action::VimGoToTop),
                (KeyCode::Char('G'), KeyModifiers::SHIFT) => Some(Action::VimGoToBottom),
                (KeyCode::Char('y'), KeyModifiers::NONE) => Some(Action::VimYank),
                (KeyCode::Char('p'), KeyModifiers::NONE) => Some(Action::VimPaste),
                (KeyCode::Char('d'), KeyModifiers::NONE) => Some(Action::VimDelete),
                (KeyCode::Char('u'), KeyModifiers::NONE) => Some(Action::VimUndo),
                (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Action::VimRedo),
                (KeyCode::Char('/'), KeyModifiers::NONE) => Some(Action::VimSearch),
                (KeyCode::Char('n'), KeyModifiers::NONE) => Some(Action::SearchNext),
                (KeyCode::Char('N'), KeyModifiers::SHIFT) => Some(Action::SearchPrevious),
                (KeyCode::Char('m'), KeyModifiers::NONE) => Some(Action::VimMark),
                (KeyCode::Char('\''), KeyModifiers::NONE) => Some(Action::VimGoToMark),
                (KeyCode::Char(' '), KeyModifiers::NONE) => Some(Action::SelectItem),
                (KeyCode::Esc, _) => Some(Action::VimEnterNormal),
                _ => None,
            },
            VimMode::Insert => match (code, modifiers) {
                (KeyCode::Esc, _) => Some(Action::VimEnterNormal),
                _ => None,
            },
            VimMode::Visual => match (code, modifiers) {
                (KeyCode::Esc, _) => Some(Action::VimEnterNormal),
                (KeyCode::Char('y'), KeyModifiers::NONE) => Some(Action::VimYank),
                (KeyCode::Char('d'), KeyModifiers::NONE) => Some(Action::VimDelete),
                (KeyCode::Char('h'), KeyModifiers::NONE) => Some(Action::NavigateLeft),
                (KeyCode::Char('j'), KeyModifiers::NONE) => Some(Action::NavigateDown),
                (KeyCode::Char('k'), KeyModifiers::NONE) => Some(Action::NavigateUp),
                (KeyCode::Char('l'), KeyModifiers::NONE) => Some(Action::NavigateRight),
                _ => None,
            },
            VimMode::Command => match (code, modifiers) {
                (KeyCode::Esc, _) => Some(Action::VimEnterNormal),
                (KeyCode::Enter, _) => Some(Action::ExecuteCommand),
                _ => None,
            },
        }
    }

    /// Enable or disable vim mode
    pub fn set_vim_mode(&mut self, enabled: bool) {
        self.vim_mode = if enabled { Some(VimMode::Normal) } else { None };
    }

    /// Change vim mode
    pub fn change_vim_mode(&mut self, mode: VimMode) {
        if self.vim_mode.is_some() {
            self.vim_mode = Some(mode);
        }
    }

    /// Get current vim mode
    pub fn get_vim_mode(&self) -> Option<VimMode> {
        self.vim_mode
    }

    /// Add custom shortcut
    pub fn add_custom_shortcut(&mut self, binding: KeyBinding, action: Action) {
        self.custom_shortcuts.insert(binding, action);
    }

    /// Remove custom shortcut
    pub fn remove_custom_shortcut(&mut self, binding: &KeyBinding) {
        self.custom_shortcuts.remove(binding);
    }

    /// Get all shortcuts
    pub fn get_all_shortcuts(&self) -> Vec<(KeyBinding, Action)> {
        let mut all = Vec::new();

        // Add standard shortcuts
        for (binding, action) in &self.shortcuts {
            all.push((binding.clone(), action.clone()));
        }

        // Add custom shortcuts
        for (binding, action) in &self.custom_shortcuts {
            all.push((binding.clone(), action.clone()));
        }

        // Sort by key binding for display
        all.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

        all
    }

    /// Get description for an action
    pub fn get_action_description(action: &Action) -> &'static str {
        match action {
            Action::NavigateUp => "Move cursor up",
            Action::NavigateDown => "Move cursor down",
            Action::NavigateLeft => "Go to parent directory",
            Action::NavigateRight => "Enter directory",
            Action::NavigatePageUp => "Page up",
            Action::NavigatePageDown => "Page down",
            Action::NavigateHome => "Go to first item",
            Action::NavigateEnd => "Go to last item",
            Action::Copy => "Copy selected files",
            Action::Move => "Move selected files",
            Action::Delete => "Delete selected files",
            Action::Rename => "Rename file",
            Action::CreateFile => "Create new file",
            Action::CreateDirectory => "Create new directory",
            Action::ViewFile => "View file",
            Action::EditFile => "Edit file",
            Action::Search => "Advanced search",
            Action::Refresh => "Refresh panels",
            Action::Help => "Show help",
            Action::Settings => "Open settings",
            Action::Quit => "Quit application",
            Action::SelectAll => "Select all files",
            Action::ToggleHidden => "Toggle hidden files",
            Action::CommandPalette => "Open command palette",
            Action::SwitchPanel => "Switch active panel",
            _ => "Action",
        }
    }
}
