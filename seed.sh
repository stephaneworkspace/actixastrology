#!/bin/sh
# Copy before the "migration" directory and file "diesel.toml" from 
# https://github.com/stephaneworkspace/city_time_zone_sqlite
DB=$(grep DATABASE_URL .env | cut -d '=' -f 2-)
rm $DB
diesel migration run
cargo run --example seed
