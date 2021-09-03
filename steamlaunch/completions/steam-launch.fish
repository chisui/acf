complete -c steam-launch -n "__fish_use_subcommand" -s s -l steam-dir -r
complete -c steam-launch -n "__fish_use_subcommand" -s u -l user -r
complete -c steam-launch -n "__fish_use_subcommand" -s p -l password -r
complete -c steam-launch -n "__fish_use_subcommand" -s x -l password-cmd -r
complete -c steam-launch -n "__fish_use_subcommand" -s h -l help -d 'Print help information'
complete -c steam-launch -n "__fish_use_subcommand" -s V -l version -d 'Print version information'
complete -c steam-launch -n "__fish_use_subcommand" -f -a "list"
complete -c steam-launch -n "__fish_use_subcommand" -f -a "start"
complete -c steam-launch -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c steam-launch -n "__fish_seen_subcommand_from list" -s h -l help -d 'Print help information'
complete -c steam-launch -n "__fish_seen_subcommand_from list" -s V -l version -d 'Print version information'
complete -c steam-launch -n "__fish_seen_subcommand_from start" -s s -l steam-dir -r
complete -c steam-launch -n "__fish_seen_subcommand_from start" -r
complete -c steam-launch -n "__fish_seen_subcommand_from start" -r
complete -c steam-launch -n "__fish_seen_subcommand_from start" -s h -l help -d 'Print help information'
complete -c steam-launch -n "__fish_seen_subcommand_from start" -s V -l version -d 'Print version information'
complete -c steam-launch -n "__fish_seen_subcommand_from help" -s h -l help -d 'Print help information'
complete -c steam-launch -n "__fish_seen_subcommand_from help" -s V -l version -d 'Print version information'
