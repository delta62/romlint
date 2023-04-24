requires = { "archive" }

function lint(file, api)
    local files = file.archive().files

    -- Some archives *are* the file, e.g. RVZ
    if files == nil then
        return
    end

    local names = table.concat(files, ", ")
    local msg = string.format("archives should contain exactly one file (saw %s)", names)
    api.assert_eq(1, #files, msg)
end
