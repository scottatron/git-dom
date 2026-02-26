use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::Cli;

pub fn run(shell: clap_complete::Shell) -> Result<()> {
    match shell {
        clap_complete::Shell::Zsh => print_zsh_completions(),
        _ => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "git-dom", &mut std::io::stdout());
        }
    }
    Ok(())
}

fn print_zsh_completions() {
    let script = r##"#compdef git-dom

_git-dom() {
    local -a subcommands
    subcommands=(
        'ls:List all submodules'
        'status:Show rich status for submodules'
        'clone:Add a submodule with Go-style path convention'
        'pull:Fetch and update submodules from upstream'
        'rm:Remove a submodule cleanly'
        'diff:Show changes across submodules'
        'foreach:Run a command in each submodule'
        'completions:Generate shell completions'
        'man:Generate or install a man page'
        'help:Print help'
    )

    _git-dom_submodule_names() {
        local -a names
        names=(${(f)"$(git config --file .gitmodules --get-regexp '^submodule\..*\.path$' 2>/dev/null | sed 's/^submodule\.\(.*\)\.path .*/\1/')"})
        _describe 'submodule' names
    }

    local curcontext="$curcontext" state line

    _arguments -C \
        '--no-colour[Disable colour output]' \
        '(-h --help)'{-h,--help}'[Print help]' \
        '(-V --version)'{-V,--version}'[Print version]' \
        '1:subcommand:->subcmd' \
        '*::arg:->args'

    case $state in
        subcmd)
            _describe 'command' subcommands
            ;;
        args)
            case $line[1] in
                ls|status)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1::name:_git-dom_submodule_names'
                    ;;
                clone)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '--no-commit[Skip commit prompt]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:url:'
                    ;;
                pull)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '--commit[Commit mode]:mode:(auto stage prompt)' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1::name:_git-dom_submodule_names'
                    ;;
                rm)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:name:_git-dom_submodule_names'
                    ;;
                diff)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '--full[Show full per-submodule diffs]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1::name:_git-dom_submodule_names'
                    ;;
                foreach)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '--parallel[Run in parallel]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:command:'
                    ;;
                completions)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:shell:(bash zsh fish powershell elvish)'
                    ;;
                man)
                    _arguments \
                        '--no-colour[Disable colour output]' \
                        '--install[Install to $XDG_DATA_HOME/man/man1 (or ~/.local/share/man/man1)]' \
                        '(-o --output)'{-o,--output}'[Write man page to path]:path:_files' \
                        '(-h --help)'{-h,--help}'[Print help]'
                    ;;
            esac
            ;;
    esac
}

_git-dom "$@"
"##;
    print!("{}", script);
}
