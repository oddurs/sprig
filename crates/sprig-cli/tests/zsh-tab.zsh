# Types a line into an interactive zsh under zpty and presses Tab twice, so
# zsh lists the candidates; prints what the terminal showed.
# Usage: zsh -f zsh-tab.zsh FPATH_DIR SPRIG_DIR WORKDIR LINE
zmodload zsh/zpty
zpty z TERM=dumb zsh -f -i
send() { zpty -w z "$1"; sleep 0.5 }
send "fpath=($1 \$fpath); PATH=$2:\$PATH; PS1='> '"
send "autoload -Uz compinit; compinit -u"
send "setopt autolist nolistbeep; cd $3"
zpty -n -w z "$4"$'\t\t'
sleep 3
local out='' chunk
while zpty -r -t z chunk; do out+=$chunk; done
print -r -- "$out"
zpty -d z
