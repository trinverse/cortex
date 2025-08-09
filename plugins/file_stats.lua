-- File Statistics Plugin
-- Provides detailed file and directory statistics

plugin = {
    name = "File Statistics",
    version = "1.0.0",
    author = "Cortex Team",
    description = "Provides detailed statistics about files and directories",
    min_cortex_version = "0.1.0",
    commands = {"stats", "analyze"},
    event_hooks = {"file_selected", "directory_changed"}
}

-- Initialize plugin
function initialize()
    cortex:show_message("File Statistics plugin initialized")
end

-- Execute plugin commands
function execute(command, args)
    if command == "stats" then
        return get_file_stats()
    elseif command == "analyze" then
        return analyze_directory()
    else
        return "Unknown command: " .. command
    end
end

-- Handle system events
function handle_event(event_name, event_data)
    if event_name == "file_selected" then
        local path = event_data.path or ""
        if path ~= "" then
            cortex:show_message("Selected: " .. path)
        end
    elseif event_name == "directory_changed" then
        local path = event_data.path or ""
        if path ~= "" then
            cortex:show_message("Changed to: " .. path)
        end
    end
end

-- Get detailed file statistics
function get_file_stats()
    local current_file = context:get_current_file()
    
    if not current_file then
        return "No file selected"
    end
    
    if not cortex:file_exists(current_file) then
        return "File does not exist: " .. current_file
    end
    
    local stats = {}
    table.insert(stats, "=== File Statistics ===")
    table.insert(stats, "File: " .. current_file)
    
    if cortex:is_directory(current_file) then
        table.insert(stats, "Type: Directory")
        local files = cortex:list_directory(current_file)
        table.insert(stats, "Items: " .. #files)
    else
        table.insert(stats, "Type: File")
        local content = cortex:read_file(current_file)
        if content then
            table.insert(stats, "Size: " .. #content .. " bytes")
            local lines = 0
            for _ in content:gmatch("[^\r\n]+") do
                lines = lines + 1
            end
            table.insert(stats, "Lines: " .. lines)
        end
    end
    
    return table.concat(stats, "\n")
end

-- Analyze directory contents
function analyze_directory()
    local current_dir = context:get_current_directory()
    local files = cortex:list_directory(current_dir)
    
    local stats = {
        total_files = 0,
        total_dirs = 0,
        extensions = {}
    }
    
    for _, file in ipairs(files) do
        local full_path = current_dir .. "/" .. file
        
        if cortex:is_directory(full_path) then
            stats.total_dirs = stats.total_dirs + 1
        else
            stats.total_files = stats.total_files + 1
            
            -- Extract extension
            local ext = file:match("%.([^%.]+)$")
            if ext then
                ext = ext:lower()
                stats.extensions[ext] = (stats.extensions[ext] or 0) + 1
            end
        end
    end
    
    local result = {}
    table.insert(result, "=== Directory Analysis ===")
    table.insert(result, "Directory: " .. current_dir)
    table.insert(result, "Files: " .. stats.total_files)
    table.insert(result, "Directories: " .. stats.total_dirs)
    table.insert(result, "")
    table.insert(result, "File Types:")
    
    for ext, count in pairs(stats.extensions) do
        table.insert(result, "  ." .. ext .. ": " .. count)
    end
    
    return table.concat(result, "\n")
end

-- Cleanup
function shutdown()
    cortex:show_message("File Statistics plugin shutting down")
end