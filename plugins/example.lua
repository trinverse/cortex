-- Example Cortex Plugin (Hot-reloadable)
-- This plugin can be modified and reloaded without restarting Cortex

plugin = {
    name = "Development Helper",
    version = "1.0.0",
    author = "Cortex Dev",
    description = "Hot-reloadable development plugin"
}

-- Called when plugin is loaded
function initialize()
    print("Development plugin loaded!")
end

-- Execute commands
function execute(command, args)
    if command == "reload" then
        return "Plugin reloaded successfully!"
    elseif command == "test" then
        return "Test command executed at " .. os.date()
    elseif command == "git-status" then
        -- Get git status of current directory
        local handle = io.popen("git status --short")
        local result = handle:read("*a")
        handle:close()
        return result
    elseif command == "build" then
        -- Trigger rebuild
        os.execute("cargo build --release &")
        return "Build started in background..."
    end
    return "Unknown command: " .. command
end

-- Called when plugin is unloaded
function shutdown()
    print("Development plugin shutting down")
end