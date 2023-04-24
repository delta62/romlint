requires = { "path" }

function contains(haystack, needle)
    for _,v in pairs(haystack) do
        if v == needle then
            return true
        end
    end

    return false
end

function lint(file, api)
    local archive = api.config.archive_format
    local ext = file.path().extension

    if !contains(archive, ext) then
        api.throw("File is not archived")
    end
end
