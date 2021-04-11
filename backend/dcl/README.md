# `dcl`

The Rust powered `dcl` (Distributed Compute Layer) for the Sybl website. This
handles incoming job requests from the `api-server` and distributes them to
clients to work on.

## Requirements

The `dcl` is written in [`Rust`](https://www.rust-lang.org) and thus requires a
Rust compiler to build. You can download the toolchain from
[here](https://www.rust-lang.org/tools/install).

Additionally, the `dcl` uses [`MongoDB`](https://www.mongodb.com/) as a
persistent datastore and [`Apache Kafka`](https://kafka.apache.org/) for
messaging, so these must be installed locally as well.

## Getting Started

The `dcl` can be built in development mode with the following command:
```bash
cargo run
```
This will download and build all the required dependencies, as well as building
the other workspace members if needed, serving the `dcl` on ports based on the
configuration options.

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

## Testing

The `dcl` tests itself both through unit tests in `src` and through integration
tests in `tests`. The integration tests require a local instance of `MongoDB`,
which can be configured in `config.toml`. These will set up the database with
some predefined data and check that specific requests succeed or fail.
