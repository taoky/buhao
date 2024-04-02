# buhao

[WIP]

A naive user-space filesystem metadata caching framework.
Designed as an alternative approach of [rsync-huai](https://github.com/tuna/rsync/blob/master/README-huai.md).

## TODO

- [x] sqlite database for server
- [ ] refresh support
- [ ] how can hook get updated metadata?
- [ ] fuse client implementation

## Architecture

Hook: A library that could be `LD_PRELOAD`ed to any (supported) program that needs to access filesystem metadata.

Client: A standalone program that directly interacts with the server.

Server: A daemon that runs on the server side, maintains database, and provides metadata to clients through UNIX socket.

Lib: Shared code.

## Testdrive

Run server:

```console
cargo run --bin buhao_server
```

Run testing client:

```console
cargo run --bin buhao_client
```

Run testing hook:

```console
$ cargo build --lib
$ # rsync
$ LD_PRELOAD=./target/debug/libbuhao_hook.so rsync --daemon --no-detach --config=assets/rsyncd-test.conf
$ # nginx
$ LD_PRELOAD=./target/debug/libbuhao_hook.so nginx -c $(pwd)/assets/nginx-test.conf
$ # kill with nginx -s stop -c $(pwd)/assets/nginx-test.conf
```

Debugging:

```console
$ sudo strace -s 65535 -f -p $(pidof rsync)  # show full syscall arguments
```
