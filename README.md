# Dodona

## Required Technology

- Rust (Use `rustup` so version of Rust compiler can be changed)
- npm
- yarn
- docker
- docker-compose (if on MacOS, this comes with Docker Desktop)

## Running Project

Use `docker-compose up`

After it spins up go to `localhost:3000` (or `{docker ip}:3000` on windows) to see the React front-end.

The API layer can be viewed at `localhost:3001/api`. You can directly use `fetch("localhost:3001/api/{whatever}")` from the React side to get data from the API.

## MongoDB

To setup the MongoDB website functionality, either have a hosted or local instance of a MongoDB database.

Once this is done, set the environment variables `CONN_STR` and `APP_NAME`. `CONN_STR` is the address of the MongoDB cluster you want to use. If using a local instance, the `CONN_STR` would be: `mongodb://localhost:27017`. This will need to change depending on which cluster is needed.

These should be set in the `.env` file. This should be created using the `env` template

## First Time

The first time this runs, it will need to download and create a number of docker images for the code to run on. This can take some time. On top of this, `react-scripts` will need to be installed so that the frontend can be run. To do this, run `yarn` in the `web` directory. This will build all the `node_modules` so they can be used by the frontend. Additionally, you may need to run `cargo install cargo-watch` in the `api-server` directory.

## Index Page
<img width="1920" alt="Sybl_Static" src="https://user-images.githubusercontent.com/37386274/85752077-041cc280-b703-11ea-8add-f29305779dea.png">

## Login Page
<img width="1920" alt="Screenshot 2020-06-25 at 16 45 51" src="https://user-images.githubusercontent.com/37386274/85752484-5e1d8800-b703-11ea-852e-695d033221de.png">
