//! `sprig mcp --instructions`: the instructions that tell a coding agent how
//! this repository plans, for AGENTS.md, CLAUDE.md or wherever an agent reads.
//!
//! `--write` keeps them in a block between two markers, so running it again
//! replaces the block and nothing else, and running it twice in a row
//! changes nothing the second time.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use crate::Result;
use crate::write::{save, write_error};

const START: &str =
    "<!-- sprig:start: written by `sprig mcp --instructions`; edits inside are replaced -->";
/// How a block starts whichever command wrote it, so a block from an older
/// version is replaced rather than joined by a second one.
const OPENS: &str = "<!-- sprig:start";
const END: &str = "<!-- sprig:end -->";

pub const INSTRUCTIONS: &str = "\
## Plans

This repository plans its work in Sprig files (`.sprig`): plain text, one item
per line, each starting with a mark: `-` to do, `~` doing, `x` done,
`/` dropped, `>` later and `?` a question. `PLAN.sprig` at the root is the
plan for the work in progress.

- Before starting, read `PLAN.sprig`, or run `sprig next` for what is ready.
- Before changing code, write the plan: add the steps as `- ` lines under the
  item they belong to, and mark the one you start as doing, `~`, by editing
  the line or with `sprig tick REF --mark ~`.
- Tick a step when it is done, with `sprig tick REF`, in the same commit as
  the work.
- A decision that belongs to a person is a `?` item that names them, such as
  `? Which payment provider?  @dana`. Do not answer it yourself; carry on with
  the work it does not block.
- Run `sprig check` before you finish, and leave no plan with errors.
- When the `sprig mcp` server is connected, use its tools rather than
  editing `.sprig` files by hand: they check each change and show the diff.
";

/// The block as it is written into a file, markers included.
fn block() -> String {
    format!("{START}\n{INSTRUCTIONS}{END}\n")
}

/// `text` with the block in it: in place of the one already there, or after
/// the rest of the file, separated by a blank line.
pub fn with_block(text: &str) -> String {
    if let (Some(s), Some(e)) = (text.find(OPENS), text.find(END))
        && s < e
    {
        let after = &text[e + END.len()..];
        let after = after
            .strip_prefix("\r\n")
            .or(after.strip_prefix('\n'))
            .unwrap_or(after);
        return format!("{}{}{after}", &text[..s], block());
    }
    if text.trim().is_empty() {
        return block();
    }
    let sep = if text.ends_with("\n\n") {
        ""
    } else if text.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{text}{sep}{}", block())
}

pub fn instructions(write: Option<&Path>) -> Result<ExitCode> {
    let Some(path) = write else {
        write!(io::stdout().lock(), "{INSTRUCTIONS}")?;
        return Ok(ExitCode::SUCCESS);
    };
    let old = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display()).into()),
    };
    let new = with_block(&old);
    let shown = path.display();
    if new == old {
        eprintln!("{shown} already has the current instructions");
    } else {
        save(path, &new).map_err(|e| write_error(path, e))?;
        eprintln!("wrote the instructions to {shown}");
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writing_twice_changes_nothing_the_second_time() {
        for start in ["", "# Agents\n\nBe careful.\n", "# Agents", "x\n\n"] {
            let once = with_block(start);
            assert_eq!(with_block(&once), once, "{start:?}");
            assert!(once.starts_with(start.trim_end()), "{start:?}");
        }
    }

    #[test]
    fn a_block_written_by_an_older_command_is_replaced_not_doubled() {
        let old = "# Agents\n\n<!-- sprig:start: written by `sprig agent`; edits inside are replaced -->\nold\n<!-- sprig:end -->\n";
        let new = with_block(old);
        assert_eq!(new.matches("sprig:start").count(), 1, "{new}");
        assert!(new.contains(START) && !new.contains("\nold\n"));
    }

    #[test]
    fn an_old_block_is_replaced_and_what_is_around_it_kept() {
        let old = format!("# Agents\n\n{START}\nold words\n{END}\n\n## Tests\n\nRun them.\n");
        let new = with_block(&old);
        assert!(!new.contains("old words"));
        assert!(new.starts_with("# Agents\n\n"));
        assert!(new.ends_with(&format!("{END}\n\n## Tests\n\nRun them.\n")));
    }
}
