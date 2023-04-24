requires = { "path", "archive" }

function contains(haystack, needle)
    for _,v in pairs(haystack) do
        local stem = string.gsub(v, "%.%w+$", "")
        if stem == needle then
            return true
        end
    end

    return false
end

function lint(file, api)
    local files = file.archive().files
    local name = file.path().stem

    if files == nil then
        return
    end

    if contains(files, name) then
        return
    end

    local msg = string.format("archived files should match their archive name (expected '%s.*')", name)

    api.throw(msg)
end
