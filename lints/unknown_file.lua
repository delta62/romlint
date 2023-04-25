requires = { "path", "file_db" }

function lint(file, api)
    local stem = file.path().stem
    local known_file = api.db_contains(stem)

    if not known_file then
        local similar_files = api.similar_files(stem)
        local msg = string.format("unrecognized file. similar files: %s", table.concat(similar_files, ", "))
        api.throw(msg)
    end
end
