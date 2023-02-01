--serialiser for VFS file
local function getBase(str)
    local parts = split(str,"/")
    local e = parts[#parts]
    return e
end

verbosity = 2
local function gen_disk(path)
    local tree = {}
    for _,fpath in pairs(fs:list(path or "")) do
        local pth = getBase(fpath)
        if fs:isdir(fpath) then
            if verbosity > 1 then print("[build] heading down to "..fpath) end
            tree[pth] = gen_disk(fpath)
        else
            if verbosity > 1 then print("[build] adding "..fpath) end
            tree[pth] = fs:read(fpath)
        end
    end
    return tree
end

local tbl = gen_disk(ARGS.directory)
fs:write(ARGS.archive,serialize(tbl))