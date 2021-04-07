# config

Minimal configuration library that allows for different settings based on the
environment. Configuration files are written in TOML syntax and can contain 4
sections (`global`, `development`, `production` and `testing`).

Based on the compilation settings, environment variables will be set using the
most specific section first, followed by `global`. This allows for different
settings to be used in production or testing, such as targetting a local
instance of MongoDB.
