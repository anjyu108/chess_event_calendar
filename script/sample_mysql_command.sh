#!/bin/bash

set -eux

password="mysql"

SCRIPT_DIR=$(cd $(dirname $0); pwd)
DATABSE_NAME="chess_info"

mysql \
  -u root \
  --password="${password}" \
  -h 127.0.0.1 \
  -e "CREATE DATABASE IF NOT EXISTS ${DATABSE_NAME} default character set utf8; SHOW DATABASES;"

mysql \
  -u root \
  --password="${password}" \
  -h 127.0.0.1 \
  "${DATABSE_NAME}" \
  < ${SCRIPT_DIR}/setup_chess_calendar_table.sql

