# Dodona

[![codecov](https://codecov.io/gh/FreddieBrown/dodona/branch/develop/graph/badge.svg?token=2LEVZ215BB)](https://codecov.io/gh/FreddieBrown/dodona)

Dodona encapsulates the frontend and API server for the business website. The
frontend is written in Vue.js and the server is written in Rust, using MongoDB
as a persistant store.

## Requirements

To run the full system, you will need the following installed:

- [Rust](https://www.rust-lang.org/learn/get-started)
- [yarn](https://yarnpkg.com/getting-started/install)
- [MongoDB](https://docs.mongodb.com/guides/server/install/)
- [Kafka](https://kafka.apache.org/quickstart)

## Getting Started

This section will walk you through getting both the frontend website and
backend server running, as well as discussing how to run a client model.

### Frontend

To start the frontend in development mode, run the following:
```bash
cd web
yarn install
yarn run serve
```
This will install all the necessary dependencies and serve the frontend,
displaying the location to view it.

### Backend

The backend contains 3 main components, being the `api-server`, `dcl` and
`analytics`. Each of these can be run by executing the following:
```bash
cd backend
cargo run --bin <binary>
```
Only the `api-server` and `dcl` are required to run models and gather
predictions.

### Mallus

`mallus` is the library that allows clients to connect to the system, receive
jobs and process them to gain credits. More information about it can be found
in its own README file as it is a submodule.

## Using the System

Sybl comes with a series of guides on using the system, including a basic
primer on machine learning, getting started with the system as a whole and how
it works. There are also client specific guides on registering as one and
adding a model.

These can be found in the `guides/` directory.

## Configuration

Configuration of the system is provided by the `config.toml` file. There is a
default one found at the root of the repository which contains some sensible
values for development. If you would like to change any of them, it's
recommended to copy it into `backend/` and make changes there, as the system
will pick the closest one it finds.

This will allow you to alter settings such as the location of MongoDB (to use
an Atlas instance for example), the name of the database to use and various
sockets that components will expect to run on.

## Local MongoDB Instances for Testing

If you want to run tests, you should use a local instance of MongoDB to avoid
deleting production data or collections that other people are using at the same
time. You can use `config.toml` to switch between a local instance and Atlas
depending on the context using something such as:

```
[global]
conn_str = <atlas_connection_string>

[testing]
conn_str = <local_connection_string>
```

Then, when you use `cargo run [--release]`, it will use the Atlas instance,
whereas using `cargo test` will use your local instance instead.
