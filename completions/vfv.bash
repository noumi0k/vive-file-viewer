# bash completion for vfv

_vfv() {
    local cur prev words cword
    _init_completion || return

    local commands="find init man help"

    case "${words[1]}" in
        find)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "-j --json -d --dir -n --limit -1 --first -t --timeout -q --quiet -c --compact -e --exact -h --help" -- "$cur"))
                    ;;
                *)
                    _filedir -d
                    ;;
            esac
            ;;
        init)
            COMPREPLY=($(compgen -W "-f --force -h --help" -- "$cur"))
            ;;
        man)
            COMPREPLY=($(compgen -W "-h --help" -- "$cur"))
            ;;
        help)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
        *)
            case "$cur" in
                -*)
                    COMPREPLY=($(compgen -W "-h --help -V --version" -- "$cur"))
                    ;;
                *)
                    COMPREPLY=($(compgen -W "$commands" -- "$cur"))
                    ;;
            esac
            ;;
    esac
}

complete -F _vfv vfv
