; Highlights for Sprig, with the capture names nvim-treesitter and Helix
; share. Each mark gets its own capture, so a theme can colour status.

(comment) @comment @spell

(title) @markup.heading
(title (escape) @string.escape)

; Status marks (§2.2).
((mark) @markup.list.unchecked (#match? @markup.list.unchecked "^-"))
((mark) @markup.list (#match? @markup.list "^~"))
((mark) @markup.list.checked (#match? @markup.list.checked "^x"))
((mark) @comment (#match? @comment "^[/>]"))
((mark) @keyword (#match? @keyword "^[?#+=]"))

; A dropped item reads as struck through; a done one recedes.
((item (mark) @_mark) @markup.strikethrough (#match? @_mark "^/"))
((item (mark) @_mark) @comment (#match? @_mark "^x"))

(answer) @markup.italic

; Tokens (§4).
(person) @variable
(tag) @tag
(anchor) @label
(priority) @punctuation.special
(after) @keyword
(field) @property
(link) @markup.link

(note) @spell
