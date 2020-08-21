#!/bin/sh
rm -rf migrations
rm diesel.toml
rm .env
git clone https://github.com/stephaneworkspace/city_time_zone_sqlite.git
cd city_time_zone_sqlite
# TODO copy json (after i remove old filter_city crate)
mv migrations ..
mv diesel.toml ..
mv .env ..
cd ..
rm -rf city_time_zone_sqlite
DB=$(grep DATABASE_URL .env | cut -d '=' -f 2-)
rm $DB
diesel migration run
cargo run --example seed
