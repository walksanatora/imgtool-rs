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
local ser = serialize(tbl)
if ARGS.compressed then
    print("Compressing")
    ser = fs:gz(ser)
end

if ARGS.sea then
    print("Self Extracting setup")
    local output
    if ARGS.compressed then
        output = 'local I,c,o,f,C,t,s,D = table.unpack({\nloadstring([=['
        output = output .. ARGS.ld
        output = output .. ']=])()\n,(function()local u,g = fs.open(shell.getRunningProgram(),"rb")g=u.readAll()u.close()return g:match("%[===%[(.+)%]===%]") end)(),shell.resolve(""),fs.open,fs.combine,type,shell.setDir,shell.dir()})\nfunction u(p,z)fs.makeDir(C(o,p))s(C(o,p))for k, v in pairs(z) do if t(v) == "table" then u(p.."/"..k,v)elseif t(v) == "string" then local h = f(fs.combine(o,C(p,k)),"wb")h.write(v)h.close()end end end u("'
        output = output .. ARGS.directory
        output = output .. '",textutils.unserialise(I:d(c)))s(o)'
        output = output .. '\n--[===['
        output = output .. ser
        output = output .. "]===]"
    else
        output = 'local c,o,f,C,t,s,D = table.unpack({\nloadstring((function()local u,g = fs.open(shell.getRunningProgram(),"rb")g=u.readAll()u.close()return g:match("%[===%[(.+)%]===%]") end)(),shell.resolve(""),fs.open,fs.combine,type,shell.setDir,shell.dir()})\nfunction u(p,z)fs.makeDir(C(o,p))s(C(o,p))for k, v in pairs(z) do if t(v) == "table" then u(p.."/"..k,v)elseif t(v) == "string" then local h = f(fs.combine(o,C(p,k)),"wb")h.write(v)h.close()end end end u("'
        output = output .. ARGS.directory
        output = output .. '",textutils.unserialise(c))s(o)'
        output = output .. '\n--[===['
        output = output .. ser
        output = output .. "]===]"
    end
    ser = output
end

fs:write(ARGS.archive,ser)
