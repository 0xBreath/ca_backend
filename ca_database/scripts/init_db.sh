#!/usr/bin/env bash

set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo 'Error: psql is not installed.' >&2
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.6.0 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

# Check if values have been set, otherwise use defaults
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
DB_NAME=${POSTGRES_DB:=test}
DB_PORT=${POSTGRES_PORT:=5432}
DB_HOST=${POSTGRES_HOST:=localhost}
PGADMIN_EMAIL=${PGADMIN_DEFAULT_EMAIL:='admin@domain.com'}
PGADMIN_PASSWORD=${PGADMIN_DEFAULT_PASSWORD:=password}

# Launch postgres in docker
if [[ -z "${SKIP_DOCKER}" ]]; then
  RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
  if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
    echo >$2 "there is a postgres container already running, kill it with"
    echo >$2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
    exit 1
  fi

  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p ${DB_PORT}:5432 \
    --name=postgres \
    --hostname=postgres \
    --network=pgnetwork \
    -d postgres \
    postgres -N 1000

  # Launch pgadmin in docker
  RUNNING_PGADMIN_CONTAINER=$(docker ps --filter 'name=pgadmin4' --format '{{.ID}}')
  if [[ -n $RUNNING_PGADMIN_CONTAINER ]]; then
    echo >$2 "there is a pgadmin container already running, kill it with"
    echo >$2 "    docker kill ${RUNNING_PGADMIN_CONTAINER}"
    exit 1
  fi

  docker run \
    -e PGADMIN_DEFAULT_EMAIL=${PGADMIN_EMAIL} \
    -e PGADMIN_DEFAULT_PASSWORD=${PGADMIN_PASSWORD} \
    -p 5050:80 \
    --network=pgnetwork \
    -d dpage/pgadmin4
fi

until PGPASSWORD=${DB_PASSWORD} psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}"

until curl -s http://localhost:5050 > /dev/null; do
  >&2 echo "Pgadmin is unavailable - sleeping"
  sleep 1
done

>&2 echo "Pgadmin is up and running on port 5050"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated."
