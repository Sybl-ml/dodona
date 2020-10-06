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

Once this is done, you can create a `.env` file at `api-server/.env` containing
the following environment variables:

```
CONN_STR=<mongodb_connection_string>
APP_NAME=<app_name>
PEPPER=<random_string>
```

These mappings will then be placed in the environment variables when the server
starts, allowing them to be used across machines and without hard-coding API
keys.

## Screenshots

### Index Page
<img width="1920" alt="Sybl_Static" src="https://user-images.githubusercontent.com/37386274/85752077-041cc280-b703-11ea-8add-f29305779dea.png">

### Login Page
<img width="1920" alt="Screenshot 2020-06-25 at 16 45 51" src="https://user-images.githubusercontent.com/37386274/85752484-5e1d8800-b703-11ea-852e-695d033221de.png">
