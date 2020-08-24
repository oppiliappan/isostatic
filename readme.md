### isostatic

> minimal url shortner

hosted instance: [isosta.tk](https://isosta.tk).

### build

```
$ crate2nix generate
$ nix-build
```

### usage

```
Usage
-----

isostatic [-h | --help] [--port <number>] [--database <path>]

Options
-------

    -h, --help       Prints help information
        --port       Port to start the server on (default: 3000)
        --database   Path to database (default: urls.db_3)
```

### logging

start with

```shell
$ RUST_LOG=isostatic=trace isostatic
```
