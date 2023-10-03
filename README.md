<p align="center">
  <a href="https://consciousnessarchive.com">
    <img alt="Consciousness Archive" src="./logo.png" />
  </a>
</p>

[//]: # (# Consciousness Archive)


# Consciousness Archive Server
Server to deliver consciousness calibrations, images, videos, audio, article/course markdown files, and more.
Connects to a PostgreSQL database to store and retrieve data.


## Database

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

Initialize Postgres Database
```shell
# brand new database
cargo make reset_database

# hust update database with migrations
cargo make update_database
```

## Run Server
```shell
cargo run -r -p ca_server
```

## Run Admin
```shell
cargo run -r -p ca_admin -t <file_type> -f <path>
```

#### Convert Evernote Article to Markdown
[evernote2md](https://github.com/wormi4ok/evernote2md)
```shell
brew install evernote2md

scripts/evernote2md.sh --input some_evernote.enex
```


### TODO
<h4 style="color: red"> High </h4>
  - Postgres bindings for calibration, courses
  - Subscription API

<h4 style="color: orange"> Medium </h4>
- Admin upload dashboard
  - input article/course as .enex file from Evernote, auto conver to .md
  - input image file, upload to google cloud storage, return url
  - input article/course title
  - upsert to database