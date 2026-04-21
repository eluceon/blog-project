#!/bin/bash -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$SENDER_PASSWORD" <<-EOSQL
  CREATE DATABASE "$POSTGRES_DB";
EOSQL
