# backend

The Rust powered backend for the Sybl website.

## Requirements

The backend is written in [`Rust`](https://www.rust-lang.org) and thus requires
a Rust compiler to build. You can download the toolchain from
[here](https://www.rust-lang.org/tools/install).

Additionally, the backend uses [`PostgreSQL`](https://www.postgresql.org/) as a
persistent datastore and [`Apache Kafka`](https://kafka.apache.org/) for
messaging, so these must be installed locally as well.

## Getting Started

There are 3 binaries found in the backend, being the `analytics`, `api-server`
and `dcl` components. These can be built in development mode using either of
the following commands
```bash
cargo run --bin <binary_name>
```
or
```bash
cd <binary_name>
cargo run
```
All other members of the workspace are used as libraries to support each other.

## Configuration

Configuration settings are handled by the `config.toml` file, which sits at the
root of the repository. It is recommended that changes are made by copying this
file to `backend` and altering it there, as this is ignored by `git`. The
following settings are available:

|      Variable       |  Type   |                         Meaning                         |
|---------------------|---------|---------------------------------------------------------|
|     `conn_str`      | string  |           The connection string for `MongoDB`           |
|   `database_name`   | string  |    The name of the database to use within `MongoDB`     |
|      `pepper`       | string  |    The additional value to use for password hashing     |
| `pbkdf2_iterations` | integer |      The number of iterations to use when hashing       |
|    `broker_port`    | integer |             The port to connect to Kafka on             |
|    `node_socket`    | integer |      The port for clients to connect to the DCL on      |
|      `health`       | integer | The number of seconds to wait between each health check |
|   `from_address`    | string  |             The email address to send from              |
|     `from_name`     | string  |              The name of the email sender               |
|   `app_password`    | string  |       The application specific password for Gmail       |
