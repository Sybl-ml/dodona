# Dodona

## Required Technology

- Rust (Use `rustup` so version of Rust compiler can be changed)
- npm
- yarn
- docker
- docker-compose (if on MacOS, this comes with Docker Desktop)

## Running Project

Use `docker-compose up`

After it spins up go to `localhost:3000` (or {docker ip}:3000 on windows) to see the react front-end.

The api layer can be viewed at `localhost:3000/api` due to the webpack-dev-server proxying setup. (which also means that you can directly use `fetch("/api/{whatever}")` from the React side and not worry about cross-origin request issues.

## First Time

The first time this runs, it will need to download and create a number of docker images for the code to run on. This can take some time. On top of this, `react-scripts` will need to be installed so that the frontend can be run. To do this, run `npm install` in the `web` directory. This will build all the `node_modules` so they can be used by the frontend. 
