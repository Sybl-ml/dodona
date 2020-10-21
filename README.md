# Dodona

Dodona encapsulates the frontend and API server for the business website. The
frontend is written in React and the server is written in Rust, using MongoDB
as a persistant store.

## Requirements

### Rust

You can install the Rust compiler and toolchain
[here](https://www.rust-lang.org/learn/get-started).

After installing the toolchain, you'll need to swap to the nightly compiler by
using `rustup default nightly`.

### JavaScript and Docker

You'll also need the following installed:

- npm
- yarn
- docker
- docker-compose

## Getting Started

When first setting up the project, you will need to go into `frontend` and run
`yarn` to download the dependencies as these will be copied into the Docker
container for now.

You can then run `docker-compose up`, which will build the Docker images for
both the frontend and the API server, before allowing you to view the frontend
at `localhost:3000`.

The API server will be running on `localhost:3001`, which means you can send
requests to it either through the console in JavaScript or using cURL for
example.

## MongoDB

To use the MongoDB website functionality, you must either have a hosted or
local instance of a MongoDB database.

Once this is done, you can create a `config.toml` file in `api-server` containing
the following basic configuration:

```
[global]
app_name = <app_name>
conn_str = <mongodb_connection_string>
pepper = <random_string>
```

These mappings will then be placed in the environment variables when the server
starts, allowing them to be used across machines and without hard-coding API
keys.

