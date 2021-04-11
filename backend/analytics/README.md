# `analytics`

The Rust powered `analytics` server for the Sybl website. This handles incoming
datasets upload messages from the `api-server` and analyses the datasets for
visualisation on the frontend.

## Requirements

The `analytics` server is written in [`Rust`](https://www.rust-lang.org) and
thus requires a Rust compiler to build. You can download the toolchain from
[here](https://www.rust-lang.org/tools/install).

Additionally, the `analytics` server uses [`MongoDB`](https://www.mongodb.com/)
as a persistent datastore and [`Apache Kafka`](https://kafka.apache.org/) for
messaging, so these must be installed locally as well.

## Getting Started

The `analytics` can be built in development mode with the following command:
```bash
cargo run
```
This will download and build all the required dependencies, as well as building
the other workspace members if needed, listening for messages from Kafka on the
socket provided (or the default of `9092` if not specified).

## Configuration

Configuration settings are handled by the `config.toml` file, which sits at the
root of the repository. It is recommended that changes are made by copying this
file to `backend` and altering it there, as this is ignored by `git`. The
following settings are available:

|      Variable       |  Type   |                         Meaning                         |
|---------------------|---------|---------------------------------------------------------|
|     `conn_str`      | string  |           The connection string for `MongoDB`           |
|   `database_name`   | string  |    The name of the database to use within `MongoDB`     |
|    `broker_port`    | integer |             The port to connect to Kafka on             |
