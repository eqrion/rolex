function $prefix$Lex(text)
{
  this._text = text;
  this._index = 0;

  this.error = false;
  this.eof = this._index >= this._text.length;
}

$prefix$Lex.prototype.dfa = [
$js-state-table$];

$prefix$Lex.prototype.getNextLexeme = function()
{
  if (this._index >= this._text.length)
  {
    this.eof = true;
    return null;
  }

  var state = this.dfa[0];
  var match = null;
  var start = this._index;
  var end = this._index;

  if (state.answer != undefined)
  {
    match = {
      type: state.answer,
      end: end,
    };
  }

  while (end < this._text.length)
  {
    var letter = this._text[end++];

    if (state[letter] == undefined)
      break;

    state = this.dfa[state[letter]];

    if (state.answer != undefined)
    {
      match = {
        type: state.answer,
        end: end,
      };
    }
  }

  if (match != null)
  {
    this._index = match.end;
    this.eof = this._index >= this._text.length;

    return {
      type: match.type,
      lexeme: this._text.substr(start, match.end - start)
    };
  }
  else
  {
    this.error = true;
    return null;
  }
};
