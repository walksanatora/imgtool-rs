--deserialiser for VFS file

function unpack_vfs(path,files)
    print("making: "..path)
    fs:makedir(path)
    for k, v in pairs(files) do
        if type(v) == "table" then
            unpack_vfs(path.."/"..k,v)
        elseif type(v) == "string" then
            print("writing:"..k)
            local fh = fs:write(path.."/"..k,v)
        end
    end
end
local contents = fs:read(ARGS.archive)
contents = contents:match("%[===%[(.+)%]===%]") or contents --extract data from sea block if it exist, if not assume it is a normal file
if ARGS.compressed then
    print("decompressed")
    contents = fs:ungz(contents)
end

unpack_vfs(
    ARGS.directory,
    loadstring("return "..contents)()
)
print("done")
