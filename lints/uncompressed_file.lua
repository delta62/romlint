requires = { "path" }

function contains(haystack, needle)
    for _,v in pairs(haystack) do
        if v == needle then
            return true
        end
    end

    return false
end

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
    local archive = api.config.archive_format
    local ext = file.path().extension
    local allowed = concat(
        api.config.obsolete_formats,
        api.config.raw_format
    )

    -- No need to check archive status of unknown files
    if not contains(allowed, ext) then
        return
    end

    if not contains(archive, ext) then
        api.throw("File is not archived")
    end
end
