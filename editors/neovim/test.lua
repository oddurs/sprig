-- Follows docs/editors (Neovim) on a clean config: loads editors/neovim/chea.lua
-- and the tree-sitter parser, opens examples/bakery/lease.chea, deletes
-- ^signed, and waits for the files that name it to report a problem.
-- Run: nvim --headless --clean -u NONE -l test.lua CHEA_BIN PARSER_SO WORKSPACE
local chea, parser, workspace = _G.arg[1], _G.arg[2], _G.arg[3]
local here = vim.fn.fnamemodify(debug.getinfo(1, 'S').source:sub(2), ':h')
vim.g.chea_command = chea
-- The guide's layout: the parser in parser/, the queries in queries/chea/.
local runtime = vim.fn.tempname()
vim.fn.mkdir(runtime .. '/parser', 'p')
vim.fn.mkdir(runtime .. '/queries/chea', 'p')
vim.loop.fs_copyfile(parser, runtime .. '/parser/chea.so')
vim.loop.fs_copyfile(here .. '/../tree-sitter-chea/queries/highlights.scm', runtime .. '/queries/chea/highlights.scm')
vim.opt.runtimepath:append(runtime)
dofile(here .. '/chea.lua')

local function fail(msg)
  io.stderr:write('neovim: ' .. msg .. '\n')
  vim.cmd('cquit 1')
end

vim.cmd('edit ' .. workspace .. '/lease.chea')
local buf = vim.api.nvim_get_current_buf()
if vim.bo[buf].filetype ~= 'chea' then fail('filetype is ' .. vim.bo[buf].filetype) end
if not vim.wait(10000, function() return #vim.lsp.get_active_clients({ bufnr = buf }) > 0 end, 50) then
  fail('the language server did not attach')
end
local ok, ts = pcall(vim.treesitter.get_parser, buf, 'chea')
if not ok or ts:parse()[1]:root():has_error() then fail('tree-sitter did not parse lease.chea') end
local q = vim.treesitter.query.get('chea', 'highlights')
if not q then fail('no highlights query') end

-- Delete "  ^signed" and save nothing: the server reads the open buffer.
local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
for i, l in ipairs(lines) do
  lines[i] = l:gsub('  %^signed', '')
end
vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

local found
if not vim.wait(15000, function()
  found = {}
  for _, d in ipairs(vim.diagnostic.get(nil)) do
    table.insert(found, vim.fn.fnamemodify(vim.api.nvim_buf_get_name(d.bufnr), ':t') .. ':' .. (d.lnum + 1) .. ' ' .. tostring(d.code))
  end
  return #found == 2
end, 100) then
  fail('expected 2 diagnostics, got: ' .. table.concat(found, ', '))
end
table.sort(found)
print('neovim: ' .. table.concat(found, ', '))
vim.cmd('qall!')
