requires = { "path" }

function lint(file, api)
    local obsolete = api.config.obsolete_formats
    local ext = file.path().extension

    for _,v in pairs(obsolete) do
        api.assert_ne(ext, v, "File is in an obsolete format")
    end
end
