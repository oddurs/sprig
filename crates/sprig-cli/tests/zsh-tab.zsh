# Types a line into an interactive zsh under zpty and presses Tab twice, so
# zsh lists the candidates; prints what the terminal showed.
# Usage: zsh -f zsh-tab.zsh FPATH_DIR SPRIG_DIR WORKDIR LINE WORD...
#
# Reads block until every WORD has appeared. `zpty -r -t` can't stand in for
# that: on macOS polling a pty reports nothing pending while output is still
# arriving, so a timed read stops after the first prompt. A watchdog ends
# the script instead if a word never comes.
zmodload zsh/zpty
{ sleep 20; kill -9 $$ } >/dev/null 2>&1 &!
local watchdog=$!
zpty z TERM=dumb zsh -f -i
send() { zpty -w z "$1"; sleep 0.5 }
send "fpath=($1 \$fpath); PATH=$2:\$PATH; PS1='> '"
send "autoload -Uz compinit; compinit -u"
send "setopt autolist nolistbeep; cd $3"
zpty -n -w z "$4"$'\t\t'
local out='' chunk w missing
while true; do
  missing=0
  for w in ${@[5,-1]}; do [[ $out == *$w* ]] || missing=1; done
  (( missing )) || break
  zpty -r z chunk || break
  out+=$chunk
  print -rn -- "$chunk"
done
zpty -d z
kill $watchdog 2>/dev/null
