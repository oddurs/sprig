-- Sprig in Neovim (0.9 or later), with nothing but Neovim itself: put this
-- file in your config, for example as ~/.config/nvim/plugin/sprig.lua.
--
-- The language server is `sprig lsp`; it gives problems as you type,
-- completion, go to definition, hover, code actions and formatting. The
-- tree-sitter parser, if you build it, gives highlighting.

vim.filetype.add({ extension = { sprig = 'sprig' } })

vim.api.nvim_create_autocmd('FileType', {
  pattern = 'sprig',
  callback = function(args)
    -- The folder that holds the plan's files is the workspace: the nearest
    -- one above with a .git, or the file's own folder.
    local file = vim.api.nvim_buf_get_name(args.buf)
    local git = vim.fs.find('.git', { path = vim.fs.dirname(file), upward = true })[1]
    vim.lsp.start({
      name = 'sprig',
      cmd = { vim.g.sprig_command or 'sprig', 'lsp' },
      root_dir = git and vim.fs.dirname(git) or vim.fs.dirname(file),
    })
    vim.bo[args.buf].commentstring = '// %s'
    -- Highlighting, when the parser is installed (see the guide).
    pcall(vim.treesitter.start, args.buf, 'sprig')
  end,
})
