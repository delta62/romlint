requires = { "path" }

function concat(...)
    local arg = {...}
    local ret = {}

    for _,t in pairs(arg) do
        for _,v in pairs(t) do
            table.insert(ret, v)
        end
    end

    return ret
end

function lint(file, api)
    local ext = file.path().extension
    local allowed = concat(
        api.config.raw_format,
        api.config.obsolete_formats,
        api.config.archive_format
    )

    local msg = string.format("unknown extension '%s'", ext)
    api.assert_contains(allowed, ext, msg)
end
