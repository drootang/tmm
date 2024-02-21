# `tmm`

`tmm` is a Textual User Interface (TUI) to manage `tmux` sessions

## Installation

## Usage

Running `tmm` will present you with a list of active tmux sessions on your current system. Use `j`/`k` or `up`/`down` to scroll through the sessions and hit `Enter` to attach the highlighted session. If you are in a tmux session already, the current session will *switch* to the selected session.

The list of sessions is searchable/filterable.

Various other actions are available through displayed hotkeys:

- Rename session
- Delete session
- Create (and optionally attach) a new named session

## NOTES

`tmm` is written in [Rust](https://www.rust-lang.org/) and uses [Ratatui](https://ratatui.rs/) to implement the TUI.
