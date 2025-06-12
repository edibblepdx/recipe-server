# recipe-server

Ethan Dibble 2025-04

Recipes are provided by [Collection of Recipes around the world](https://www.kaggle.com/datasets/prajwaldongre/collection-of-recipes-around-the-world/data) Kaggle dataset under the public domain.

# Build and Run

By default the recipe database URI is `sqlite://db/recipe.db`. You can override this with the `--db-uri` command-line argument. If using the default path, install `sqlx-cli` and create a `db` directory in the root of the project and create a database file with `sqlx database create`. Then apply pending migrations with `sqlx migrate run`.

To build and run this code for the first time, you will probably want:

`cargo run --release -- --init-from assets/static/receipes_from_around_the_world.csv`

This will load an initial collection of recipes into the newly-created database.

> Warning that some of the recipes in the dataset are malformed and will not load properly.

You can change the port with `--port [number]`

## Routes

The router merges web and api routes. 

Web routes return html and never fail. They accept `id` and `cuisine` parameters and will just return a default stub if not found.

Api routes return a json view of data and can fail with a `404` response. You can find api documentation at `/redoc`. They allow for get requests by `id`, `cuisine`, and `random`.

## Development

For working on the code, you will want to

```
cargo install sqlx-cli`
```

* `sqlx` migrations are turned on, with reverse sequential migrations. Add a            migration called <name> with
  ```
  sqlx migrate add -r -s <name>
  ```
  and then edit the migration files.

* `sqlx` compile-time checking of queries against the database schemas is turned on.    If you modify the database schemas or the queries in the source code, please run
  ```
  sqlx prepare
  ```
  to update things so that users can compile before the database is built and   migrated.
 
Because of the above you may need to
```
git add .sqlx migrations
```
before committing to ensure things are up to date.
# License

This work is made available under the "MIT License". See the file `LICENSE.txt` in this distribution for license terms.
