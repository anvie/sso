Web Single-sign-on framework
=============================

Single-sign-on framework written in Rust.

**Features**:

1. Open LDAP integration.
2. Embedded/Stand-alone session store backed by RocksDB.
3. Web UI interface for login and "soon" for access control management.
4. Restful API.

**Ingredients**:

* Open LDAP
* Nickel
* Semantic UI
* RocksDB

Development
-------------

**Prerequisites**

1. Rust nightly.
2. Open LDAP.
3. Inotify (for hot-reload/auto-compile).

For first init, please type:

    $ make init-dev

Run `./etc/script/devmon.sh` for hot reloading when template files or sources modified.

For compile only (no re-run) when source changes add `--compile-only` parameter:

    ./etc/script/devmon.sh --compile-only
