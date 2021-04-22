# Deployment

This document contains information about the deployment instance of the system,
including how things operate and useful tips for debugging issues.

The deployed instance lives at `sybl.tech` and runs on a Digital Ocean server
located in London that is owned by @AlistairRobinson.

## Logging In

To login to the server (once you have access), simply run `ssh root@sybl.tech`.

## General Structure

There are 3 main directories on the system that are important:

- `/root`
- `/var/log`
- `/etc`

### `/root`

This directory contains the Git repositories for the project (`dodona`) and the
continuous deployment (`fisherman`). No real changes exist in `dodona` in terms
of production versus local development.

### `/var/log`

This contains all of the log files for various components of the system.
Typically there are both a `<binary>.out.log` and `<binary>.err.log` file,
which capture outputs to `stdout` and `stderr` respectively. The Rust backend
will mostly log to `stdout`, other than any `panic` messages which will be
placed into the `err` file.

One useful command for viewing logs is `tail -f <logfile>`, which will show you
the last few lines and then follow any updates. This allows you to watch the
events of the system while you make changes on the frontend.

### `/etc`

This contains the configuration files for `nginx` and `supervisord`. `nginx`
acts as a proxy for the API Server and DCL and redirects messages to the
necessary places. `supervisord` is used to monitor the running processes on the
system (`api-server`, `dcl`, `analytics` and so on) and restart them if needed.
Changes to configuration files are rarely needed.

## `supervisord`

`supervisord` is a daemon process that watches the other binaries on the
system, redirecting their `stdout` and `stderr` outputs to the files specified
in their configuration (found at `/etc/supervisor/conf.d/dcl.conf` for
example).

Interacting with `supervisord` is done through the `supervisorctl` command line
program. The main commands are as follows:

- `start`: begin executing a program
- `stop`: stop executing a program
- `restart`: restart execution of a program (useful if it has been rebuilt or
  configuration has changed)

So, to restart the `api-server` after rebuilding it, all you need to do is run
`supervisorctl restart api-server` and it will handle stopping the existing
instance and using your new binary instead.

## Updating Production

Usually this is done by `fisherman` automatically on changes to `develop`, but
if that is broken then you can perform the update yourself.

### Building the Frontend

The frontend can be built by going to `/root/dodona/web` and running the
following commands:

```bash
yarn install
yarn build
cp -r dist/* /usr/share/nginx/html
```

This will install any new dependencies, build the frontend in production mode
and then copy the relevant files into the `nginx` directory. Building in
production mode takes around 25 seconds to complete usually.

### Building the Backend

The backend is comparatively easier to get working. Simply `cd
/root/dodona/backend` and run the following commands, substituting for the
appropriate binary you would like to rebuild:

```bash
cargo build --release --bin <binary>
supervisorctl restart <binary>
```

This can take a while, with the `api-server` taking approximately 2 minutes to
rebuild sometimes and using graceful shutdown so just wait it out.

## Increasing Log Output

All of the `api-server`, `analytics` and `dcl` cap their logging levels at
`debug` to prevent the log files becoming too crowded. However, the `dcl`
especially has some statements at the `trace` level, which are even more
fine-grained. These can be enabled by editing `backend/dcl/src/main.rs` and
changing the value for the `dcl` to `LogLevel::Trace`. You can then rebuild and
restart it to get the `trace` level logs.
