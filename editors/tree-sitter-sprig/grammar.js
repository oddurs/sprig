/**
 * A line-level grammar for Sprig (spec/sprig.md).
 *
 * Each line is classified the way §2.1 classifies it, and an item's tokens
 * are lifted the way §4 lifts them. Nesting is not tracked: indentation is
 * relative (§3.1), which a context-free grammar cannot follow without an
 * external scanner, and the tree an editor shows comes from the language
 * server instead. What this grammar gets right is every line's kind and
 * every token's, which is what highlighting needs.
 *
 * Tokens are told apart as the core tells them apart: a mark only counts
 * with whitespace after it, so marks win over note text by precedence; a
 * token only counts when it is the whole word, so `@jo,` is a word because
 * the longer match wins; and where two matches are equally long, the token
 * defined first wins, so `@jo` is a person rather than a word.
 */

// A field's value (§4.6): a quoted run, or anything but a leading `//`.
const VALUE = /("[^"\n]*"\S*|[^\s\/"]\S*|\/[^\s\/]\S*|\/|"\S*)/.source;

// The rest of a line: tokens and the whitespace between them, possibly none.
const content = ($) => repeat(choice($._token, $._ws));

module.exports = grammar({
  name: 'sprig',

  // Whitespace is meaningful: it ends tokens and indents lines.
  extras: (_) => [],

  // The end of a line with something on it, which may be the end of the
  // file (src/scanner.c).
  externals: ($) => [$._eol],

  rules: {
    // Blank lines and comments, then the first line that says something:
    // the title when it is not indented and not an item (§1.2).
    source_file: ($) => seq(repeat($._preamble), optional(seq($._first, repeat($._line)))),

    _preamble: ($) => choice($._blank_line, $._comment_line),
    _first: ($) => choice($.title, $._item_line, $._answer_line, $._indented_line),
    _line: ($) =>
      choice(
        $._blank_line,
        $._comment_line,
        $._item_line,
        $._answer_line,
        $._escaped_line,
        $._note_line,
      ),

    title: ($) => seq(optional(alias(token(prec(3, '\\')), $.escape)), $._token, content($), $._eol),

    _blank_line: ($) => seq(optional($._ws), $._nl),
    _comment_line: ($) => seq(optional($._ws), $.comment, $._eol),
    _item_line: ($) => seq(optional($._ws), $.item),
    _answer_line: ($) => seq(optional($._ws), $.answer),
    _escaped_line: ($) => seq(optional($._ws), $.escaped_note, $._eol),
    _note_line: ($) => seq(optional($._ws), $.note, $._eol),
    // An indented first line is never the title (§1.3).
    _indented_line: ($) =>
      seq($._ws, choice($.escaped_note, $.note), $._eol),

    // §2.2: a mark is a mark only with whitespace or the end of the line after it.
    item: ($) =>
      choice(
        seq(field('mark', $.mark), content($), $._eol),
        field('mark', alias($._bare_mark, $.mark)),
      ),
    mark: (_) => token(prec(2, /[-~x\/>?#+][^\S\r\n]+/)),
    // A mark alone on its line takes the line ending with it, so that the
    // `x` of `x-ray` cannot be one.
    _bare_mark: (_) => token(prec(2, /[-~x\/>?#+]\r?\n/)),

    answer: ($) =>
      choice(
        seq(field('mark', alias($._answer_mark, $.mark)), content($), $._eol),
        field('mark', alias($._bare_answer, $.mark)),
      ),
    _answer_mark: (_) => token(prec(2, /=[^\S\r\n]+/)),
    _bare_answer: (_) => token(prec(2, /=\r?\n/)),

    comment: (_) => token(prec(3, /\/\/[^\r\n]*/)),
    escaped_note: (_) => token(prec(2, /\\[^\r\n]*/)),
    note: (_) => token(prec(0, /[^\s][^\r\n]*/)),

    _token: ($) => choice($.person, $.tag, $.anchor, $.priority, $.after, $.field, $.link, $.word),

    person: (_) => /@[\p{L}\p{N}_.\-]+/,
    tag: (_) => /#[\p{L}\p{N}_\-]+/,
    anchor: (_) => /\^[\p{L}\p{N}_\-]+/,
    priority: (_) => /!{1,3}/,
    after: (_) => new RegExp('[aA][fF][tT][eE][rR]:' + VALUE),
    field: (_) => new RegExp('\\p{L}[\\p{L}\\p{N}_\\-]*:' + VALUE),
    link: (_) => /\[\[[^\]\s]+\]\]/,
    // At most one quoted run, opened before the first whitespace (§4.1).
    word: (_) => /[^\s"]*"[^"\r\n]*"\S*|\S+/,

    _ws: (_) => /[^\S\r\n]+/,
    _nl: (_) => /\r?\n/,
  },
});
