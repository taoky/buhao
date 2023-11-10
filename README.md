# buhao

[WIP]

A naive user-space filesystem metadata caching framework.
Designed as an alternative approach of [rsync-huai](https://github.com/tuna/rsync/blob/master/README-huai.md).

## Architecture

Hook: A library that could be `LD_PRELOAD`ed to any (supported) program that needs to access filesystem metadata.

Client: A standalone program that directly interacts with the server.

Server: A daemon that runs on the server side, maintains database, and provides metadata to clients through UNIX socket.

Lib: Shared code.
