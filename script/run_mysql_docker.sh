#!/bin/bash

set -eux

echo "DANGEROUS!! : MUST Change MYSQL_ROOT_PASSWORD for product environment"

docker run -it \
  --name chess_calendar_mysql \
  -e BIND-ADDRESS=0.0.0.0 \
  -e MYSQL_ROOT_PASSWORD=mysql \
  -p 3306:3306 \
  -d \
  mysql:5.6
