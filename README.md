# act-local-cache-server

Local artifact/cache server for use with [nektos/act](https://github.com/nektos/act).

## Protocols implemented

* [actions/cache@v3](https://github.com/actions/cache)
* [actions/upload-artifact@v3](https://github.com/actions/upload-artifact)
* [actions/download-artifact@v3](https://github.com/actions/download-artifact)

## Installation

[Rust toolchain](https://rustup.rs/) is required.

```console
$ cargo install act-local-cache-server --git https://github.com/ocxtal/act-local-cache-server
```

## Usage

Save the following `.actrc` to the root of the repository you run nektos/act.

```
--env ACTIONS_CACHE_URL=http://127.0.0.1:8000/
--env ACTIONS_RUNTIME_URL=http://127.0.0.1:8000/
--env ACTIONS_RUNTIME_TOKEN=token
```

and launch the server in the directory where you want to save artifacts and caches.

```console
$ export ACT_LOCAL_CACHE_SERVER_TOKEN=token
$ act-local-cache-server --address=127.0.0.1 --port=8000
```

It creates `.act_local_cache/{artifacts,caches}` there for artifacts and caches, respectively.

## Copyright and License

Hajime Suzuki (2022). Licensed under MIT.