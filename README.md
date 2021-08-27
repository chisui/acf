[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![LICENSE](https://img.shields.io/badge/license-apache-blue.svg)](LICENSE-APACHE)
[![Crates.io Version](https://img.shields.io/crates/v/steam-acf.svg)](https://crates.io/crates/steam-acf)

# Steam acf

Tool to convert Steam `.acf` files to JSON.

## usage

```
USAGE:
    acf [FLAGS] [OPTIONS] <file>

FLAGS:
    -c, --compact    compact instead of pretty-printed output
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --indent <indent>    how many spaces should be used per indentation step [default: 2]

ARGS:
    <file>
```

## examples 

```
$ acf ~/.local/share/Steam/steamapps/appmanifest_632470.acf
{
  "AppState": {
    "appid": "632470",
    "Universe": "1",
    "name": "Disco Elysium",
...
```

This can be used together with [`jq`](https://stedolan.github.io/jq/) like this:

```
$ acf -c ~/.local/share/Steam/steamapps/appmanifest_632470.acf | jq .AppState.name
"Disco Elysium"
```

Notice the `-c` flag to pass the JSON without extreneous whitespace.
