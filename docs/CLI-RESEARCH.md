# CLI patterns for the Syllaslop Console

This survey covers command-line shells and editors, command palettes,
history tools, and system inspection TUIs. The implementation borrows
interaction patterns through the native console registry and input path.

| References | Application to SLC |
| --- | --- |
| [OpenCode v2 CLI bindings](https://opencode.ai/docs/cli/keybinds/) and [Helix commands](https://docs.helix-editor.com/command-line.html) | Ctrl+P opens one searchable command finder. Escape restores the draft. Selection stages a command for editing. |
| [fish interactive use](https://fishshell.com/docs/current/interactive.html) and [PSReadLine predictions](https://learn.microsoft.com/en-us/powershell/scripting/learn/shell/using-predictors) | Put the proposed suffix directly after typed input and accept with Right. Use guest history and the command registry as data sources. |
| [Readline](https://tiswww.case.edu/php/chet/readline/rltop.html), [fzf](https://github.com/junegunn/fzf#key-bindings-for-command-line), and [Atuin](https://docs.atuin.sh/cli/guide/search/) | Use Ctrl+R for the current boot's history. Filter and stage commands in place. Local memory holds this boot's entries. |
| [Zsh completion](https://zsh.sourceforge.io/Doc/Release/Completion-System.html) and [Vim completion](https://vimhelp.org/cmdline.txt.html#cmdline-completion) | Retain Tab for argument and command completion. Search ranks exact names, prefixes, then ordered letter matches, then descriptions. |
| [Nushell number formatting](https://www.nushell.sh/commands/docs/format_number.html) | Surface decimal, hex, octal and binary together. Provide a small checked integer evaluator inside SLC. |
| [btop](https://github.com/aristocratos/btop) and [htop](https://htop.dev/) | Start with guest values that can be verified. `memmap` displays the loader's actual physical ranges on demand. |
| [lazygit](https://github.com/jesseduffield/lazygit) and [bat](https://github.com/sharkdp/bat) | Adopt keyboard discovery and color that signals content. Keep one input field and one set of contextual hints. |

SLC uses one two-tone wordmark and one filled input surface. Search replaces
the working area with selectable rows, while output and inspection stay in the
same screen column. There are no animation timers or visual transitions.
The previous OpenCode v2 source audit remains in
[OPENCODE-V2-RESEARCH.md](OPENCODE-V2-RESEARCH.md).
