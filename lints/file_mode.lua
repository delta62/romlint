requires = { "stat" }

function lint(file, api)
    local stat = file.stat()
    local assert = api.assert

    if stat.is_dir then
        assert(
            "mode",
            stat.mode == 0755,
            "incorrect permissions",
            "directories must have mode 755"
        )
    elseif stat.is_file then
        assert(
            "mode",
            stat.mode == 0644,
            "incorrect permissions",
            "files must have mode 644"
        )
    end
end
