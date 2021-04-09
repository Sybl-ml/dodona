# `models`

Defines the structure of collections within the database. This includes basic
types such as `User`, `Project` and `DatasetDetails`, but also defines the
structure of `JobConfiguration`s and implements a basic form of MongoDB's
GridFS, which allows large files to be stored.

## Testing

Like the `api-server` and `dcl`, testing this library requires a local instance
of MongoDB which can be set up in `config.toml`.
