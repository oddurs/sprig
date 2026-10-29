// The end of a line that has something on it: a line ending, or the end of
// the file, since a file need not end with a newline (spec §1.1). A regex
// cannot match the end of input, so this one token is scanned by hand.
//
// Blank lines keep the plain newline token: a zero-width end at the end of
// the file is only ever offered after a line's content, so it cannot repeat.

#include "tree_sitter/parser.h"

enum TokenType { EOL };

void *tree_sitter_sprig_external_scanner_create(void) { return NULL; }
void tree_sitter_sprig_external_scanner_destroy(void *payload) { (void)payload; }
unsigned tree_sitter_sprig_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}
void tree_sitter_sprig_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

bool tree_sitter_sprig_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[EOL]) {
    return false;
  }
  lexer->result_symbol = EOL;
  if (lexer->eof(lexer)) {
    return true;
  }
  if (lexer->lookahead == '\r') {
    lexer->advance(lexer, false);
  }
  if (lexer->lookahead == '\n') {
    lexer->advance(lexer, false);
    return true;
  }
  return false;
}
