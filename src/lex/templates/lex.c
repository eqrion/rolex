#include <string.h>

typedef struct
{
  const char *text;
  unsigned long index;
  unsigned long length;

  int eof;
  int error;
} $prefix$lexer;

enum
{
$c-token-enum$};

char $prefix$lexer_table[][256] = {
$c-state-table$};
long $prefix$lexer_answer[] = { $c-answer-table$ };

void $prefix$lexer_init($prefix$lexer *lex, const char *text)
{
  lex->text = text;
  lex->index = 0;
  lex->length = strlen(text);

  lex->eof = lex->index >= lex->length;
  lex->error = 0;
}

typedef struct
{
  long        type;
  const char      *lexeme;
  unsigned long     lexeme_length;
} $prefix$lexeme;

int $prefix$lexer_next_lexeme($prefix$lexer *lex, $prefix$lexeme *out)
{
  if (lex->index >= lex->length)
  {
    lex->eof = 1;

    $prefix$lexeme res = { 0, 0, 0 };
      *out = res;

      return 0;
  }

  unsigned long i = 0;
  unsigned long start = lex->index;
  unsigned long end = lex->index;

  long      match_type = -1;
  unsigned long   match_end = -1;

  if ($prefix$lexer_answer[i] != -1)
  {
    match_type = $prefix$lexer_answer[i];
    match_end = end;
  }

  while (end < lex->length)
  {
    char letter = lex->text[end++];

    if ($prefix$lexer_table[i][letter] == -1)
      break;

    i = $prefix$lexer_table[i][letter];

    if ($prefix$lexer_answer[i] != -1)
    {
      match_type = $prefix$lexer_answer[i];
      match_end = end;
    }
  }

  if (match_type != -1)
  {
    lex->index = match_end;
    lex->eof = lex->index >= lex->length;

    $prefix$lexeme res = {
      match_type,
      lex->text + start,
      match_end - start
    };
    *out = res;

      return 1;
  }
  else
  {
    lex->error = 1;

    $prefix$lexeme res = { 0, 0, 0 };
      *out = res;

    return 0;
  }
}
