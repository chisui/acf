
use builtin;
use str;

edit:completion:arg-completer[steam-launch] = [@words]{
    fn spaces [n]{
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand [text desc]{
        edit:complex-candidate $text &display-suffix=' '(spaces (- 14 (wcswidth $text)))$desc
    }
    command = 'steam-launch'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        command = $command';'$word
    }
    completions = [
        &'steam-launch'= {
            cand -s 's'
            cand --steam-dir 'steam-dir'
            cand -u 'u'
            cand --user 'user'
            cand -p 'p'
            cand --password 'password'
            cand -x 'x'
            cand --password-cmd 'password-cmd'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand list 'list'
            cand start 'start'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'steam-launch;list'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'steam-launch;start'= {
            cand -s 's'
            cand --steam-dir 'steam-dir'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'steam-launch;help'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
    ]
    $completions[$command]
}
