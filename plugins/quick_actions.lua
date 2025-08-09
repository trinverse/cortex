-- Quick Actions Plugin
-- Provides common file operations and shortcuts

plugin = {
    name = "Quick Actions",
    version = "1.0.0", 
    author = "Cortex Team",
    description = "Common file operations and productivity shortcuts",
    min_cortex_version = "0.1.0",
    commands = {"backup", "duplicate", "open-with", "compress"},
    event_hooks = {"command_executed"}
}

function initialize()
    cortex:show_message("Quick Actions plugin loaded")
end

function execute(command, args)
    if command == "backup" then
        return backup_file()
    elseif command == "duplicate" then
        return duplicate_file()
    elseif command == "open-with" then
        return open_with_external(args[1] or "")
    elseif command == "compress" then
        return compress_files()
    else
        return "Unknown command: " .. command
    end
end

function handle_event(event_name, event_data)
    if event_name == "command_executed" then
        local cmd = event_data.command or ""
        if cmd == "delete" then
            cortex:show_message("Remember: Deleted files can't be recovered!")
        end
    end
end

function backup_file()
    local current_file = context:get_current_file()
    
    if not current_file then
        return "No file selected"
    end
    
    if cortex:is_directory(current_file) then
        return "Cannot backup directories (use compress instead)"
    end
    
    local content = cortex:read_file(current_file)
    if not content then
        return "Could not read file: " .. current_file
    end
    
    local backup_path = current_file .. ".backup"
    local counter = 1
    
    -- Find available backup name
    while cortex:file_exists(backup_path) do
        backup_path = current_file .. ".backup" .. counter
        counter = counter + 1
    end
    
    if cortex:write_file(backup_path, content) then
        return "Backup created: " .. backup_path
    else
        return "Failed to create backup"
    end
end

function duplicate_file()
    local current_file = context:get_current_file()
    
    if not current_file then
        return "No file selected"
    end
    
    if cortex:is_directory(current_file) then
        return "Cannot duplicate directories"
    end
    
    local content = cortex:read_file(current_file)
    if not content then
        return "Could not read file: " .. current_file
    end
    
    local duplicate_path = current_file .. "_copy"
    local counter = 1
    
    -- Find available duplicate name
    while cortex:file_exists(duplicate_path) do
        duplicate_path = current_file .. "_copy" .. counter
        counter = counter + 1
    end
    
    if cortex:write_file(duplicate_path, content) then
        return "File duplicated: " .. duplicate_path
    else
        return "Failed to duplicate file"
    end
end

function open_with_external(app)
    local current_file = context:get_current_file()
    
    if not current_file then
        return "No file selected"
    end
    
    if app == "" then
        app = "open"  -- Default macOS command
    end
    
    local cmd = app .. " '" .. current_file .. "'"
    local result = cortex:execute_command(cmd)
    
    if result then
        return "Opened with " .. app .. ": " .. current_file
    else
        return "Failed to open with " .. app
    end
end

function compress_files()
    local selected_files = context:get_selected_files()
    local current_dir = context:get_current_directory()
    
    if #selected_files == 0 then
        local current_file = context:get_current_file()
        if current_file then
            selected_files = {current_file}
        else
            return "No files selected"
        end
    end
    
    local archive_name = "archive.zip"
    local counter = 1
    
    while cortex:file_exists(current_dir .. "/" .. archive_name) do
        archive_name = "archive" .. counter .. ".zip"
        counter = counter + 1
    end
    
    local files_str = ""
    for _, file in ipairs(selected_files) do
        files_str = files_str .. " '" .. file .. "'"
    end
    
    local cmd = "zip -r '" .. current_dir .. "/" .. archive_name .. "'" .. files_str
    local result = cortex:execute_command(cmd)
    
    if result and result ~= "" then
        return "Files compressed to: " .. archive_name .. "\n" .. result
    else
        return "Compression completed: " .. archive_name
    end
end

function shutdown()
    cortex:show_message("Quick Actions plugin unloaded")
end