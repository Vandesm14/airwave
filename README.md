# Airwave

## Server

The first and main back-end of the game.

Run the server:

```bash
cargo run --bin server
```

You can change the log level by setting `RUST_LOG` to one of the following,
which will show all logs for that level and above:

- error
- warn
- info
- debug
- trace

## Web Client

The first and most feature-complete front-end.

It is recommended to use [pnpm] as this uses their workspace system. Though it
should still be possible to build using other package managers too.

Install the dependencies, build the client, and then serve the website:

```bash
pnpm i
pnpm client-web:build
cargo run --bin serve client-web/dist
```

## Serve

A simple website server that hosts a directory.

Simply run the `serve` binary with the directory to serve, with an optional
address to use:

```bash
cargo run --bin serve path/to/directory 127.0.0.1:8080
```

You can change the log level by setting `RUST_LOG` to one of the following,
which will show all logs for that level and above:

- error
- warn
- info
- debug
- trace

[pnpm]: https://pnpm.io
