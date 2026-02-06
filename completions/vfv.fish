# fish completion for vfv

# Disable file completion by default
complete -c vfv -f

# Main commands
complete -c vfv -n "__fish_use_subcommand" -a "find" -d "Fuzzy search files and directories"
complete -c vfv -n "__fish_use_subcommand" -a "init" -d "Initialize config, shell completions, and man page"
complete -c vfv -n "__fish_use_subcommand" -a "man" -d "Generate man page"
complete -c vfv -n "__fish_use_subcommand" -a "help" -d "Print help"

# Global options
complete -c vfv -n "__fish_use_subcommand" -s h -l help -d "Print help"
complete -c vfv -n "__fish_use_subcommand" -s V -l version -d "Print version"

# find subcommand
complete -c vfv -n "__fish_seen_subcommand_from find" -s j -l json -d "Output as JSON"
complete -c vfv -n "__fish_seen_subcommand_from find" -s d -l dir -d "Search directories only"
complete -c vfv -n "__fish_seen_subcommand_from find" -s n -l limit -d "Maximum number of results" -x
complete -c vfv -n "__fish_seen_subcommand_from find" -s 1 -l first -d "Output only the top result"
complete -c vfv -n "__fish_seen_subcommand_from find" -s t -l timeout -d "Timeout in seconds" -x
complete -c vfv -n "__fish_seen_subcommand_from find" -s q -l quiet -d "Quiet mode (no spinner)"
complete -c vfv -n "__fish_seen_subcommand_from find" -s c -l compact -d "Compact JSON output"
complete -c vfv -n "__fish_seen_subcommand_from find" -s e -l exact -d "Exact match (no fuzzy)"
complete -c vfv -n "__fish_seen_subcommand_from find" -s h -l help -d "Print help"

# init subcommand
complete -c vfv -n "__fish_seen_subcommand_from init" -s f -l force -d "Overwrite existing files"
complete -c vfv -n "__fish_seen_subcommand_from init" -s h -l help -d "Print help"

# man subcommand
complete -c vfv -n "__fish_seen_subcommand_from man" -s h -l help -d "Print help"
