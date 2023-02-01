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

unpack_vfs(
    ARGS.directory,
    loadstring("return "..fs:read(ARGS.archive))())
print("done")