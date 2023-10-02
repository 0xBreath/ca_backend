<p align="center">
  <a href="https://consciousnessarchive.com">
    <img alt="Consciouness Archive" src="./logo.png" width="250" />
  </a>
</p>

[//]: # (# Consciousness Archive)


# Consciousness Archive Server
Server to deliver consciousness calibrations, images, videos, audio, article/course markdown files, and more.
Connects to a PostgreSQL database to store and retrieve data.


## Setup

Install `cargo make` to easily manage the server and docker container.
```bash
cargo install cargo-make
```

Install PostgreSQL to start database server and create superuser
```bash
# For MacOS
cargo make install_postgresql_macos && cargo make start_postgresql_macos
# For Linux
cargo make install_postgresql_linux && cargo make start_postgresql_linux

# If getting the error: 
# psql: error: connection to server on socket "/tmp/.s.PGSQL.5432" failed: FATAL
# run this to debug:
rm /opt/homebrew/var/postgresql@13/postmaster.pid
brew services restart postgresql@13

# create supseruser
createuser -s postgres
# check that superuser exists
psql -U postgres -c "SELECT * FROM pg_user;"
psql -U postgres

# quit psql shell
\q
```

Create New Postgres Database
```shell
cargo make reset_database
```

Just update database with migrations
```shell
cargo make update_database
```