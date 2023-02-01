--util.lua
-- everything here is loaded (then sandboxed) into _G
function clone(obj, seen)
    -- Handle non-tables and previously-seen tables.
    if type(obj) ~= 'table' then return obj end
    if seen and seen[obj] then return seen[obj] end
    -- New table; mark it as seen and copy recursively.
    local s = seen or {}
    local res = {}
    s[obj] = res
    for k, v in pairs(obj) do res[clone(k, s)] = clone(v, s) end
    return setmetatable(res, getmetatable(obj))
end
local g_tLuaKeywords = {
    ["and"] = true,
    ["break"] = true,
    ["do"] = true,
    ["else"] = true,
    ["elseif"] = true,
    ["end"] = true,
    ["false"] = true,
    ["for"] = true,
    ["function"] = true,
    ["if"] = true,
    ["in"] = true,
    ["local"] = true,
    ["nil"] = true,
    ["not"] = true,
    ["or"] = true,
    ["repeat"] = true,
    ["return"] = true,
    ["then"] = true,
    ["true"] = true,
    ["until"] = true,
    ["while"] = true,
}
local function serialize_impl(t, tracking, indent, opts)
    local sType = typeof(t)
    if sType == "table" then
        if tracking[t] ~= nil then
            if tracking[t] == false then
                return ""
            else
                return ""
            end
        end
        tracking[t] = true

        local result
        if next(t) == nil then
            -- Empty tables are simple
            result = "{}"
        else
            -- Other tables take more work
            local open, sub_indent, open_key, close_key, equal, comma = "{\n", indent .. "  ", "[ ", " ] = ", " = ", ",\n"
            if opts.compact then
                open, sub_indent, open_key, close_key, equal, comma = "{", "", "[", "]=", "=", ","
            end

            result = open
            local seen_keys = {}
            for k, v in ipairs(t) do
                seen_keys[k] = true
                result = result .. sub_indent .. serialize_impl(v, tracking, sub_indent, opts) .. comma
            end
            for k, v in pairs(t) do
                if not seen_keys[k] then
                    local sEntry
                    if type(k) == "string" and not g_tLuaKeywords[k] and string.match(k, "^[%a_][%a%d_]*$") then
                        sEntry = k .. equal .. serialize_impl(v, tracking, sub_indent, opts) .. comma
                    else
                        sEntry = open_key .. serialize_impl(k, tracking, sub_indent, opts) .. close_key .. serialize_impl(v, tracking, sub_indent, opts) .. comma
                    end
                    result = result .. sub_indent .. sEntry
                end
            end
            result = result .. indent .. "}"
        end

        if opts.allow_repetitions then
            tracking[t] = nil
        else
            tracking[t] = false
        end
        return result

    elseif sType == "string" then
        return string.format("%q", t)

    elseif sType == "number" then
        if t ~= t then --nan
            return "0/0"
        elseif t == math.huge then
            return "1/0"
        elseif t == -math.huge then
            return "-1/0"
        else
            return tostring(t)
        end

    elseif sType == "boolean" or sType == "nil" then
        return tostring(t)

    else
        return ("??"..sType.."??")

    end
end
function serialize(t, opts)
    local tTracking = {}

    if opts then
        --field(opts, "compact", "boolean", "nil")
        --field(opts, "allow_repetitions", "boolean", "nil")
    else
        opts = {}
    end
    return serialize_impl(t, tTracking, "", opts)
end

function split(inputstr, sep)
    if sep == nil then
            sep = "%s"
    end
    local t={}
    for str in string.gmatch(inputstr, "([^"..sep.."]+)") do
            table.insert(t, str)
    end
    return t
end