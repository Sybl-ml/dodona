# Dodona

## Running Project

Use `docker-compose up`

After it spins up go to `localhost:3000` (or {docker ip}:3000 on windows) to see the react front-end.

The api layer can be viewed at `localhost:3000/api` due to the webpack-dev-server proxying setup. (which also means that you can directly use `fetch("/api/{whatever}")` from the React side and not worry about cross-origin request issues.
