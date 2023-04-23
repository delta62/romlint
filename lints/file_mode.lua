requires = { "stat" }

function lint(file, api)
    local stat = file.stat()
    local assert_eq = api.assert_eq

    if stat.is_dir then
        -- Octal 755
        assert_eq(493, stat.mode, "directories must have mode 755")
    elseif stat.is_file then
        -- Octal 644
        assert_eq(420, stat.mode, "files must have mode 644")
    end
end
