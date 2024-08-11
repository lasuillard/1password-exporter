#!/usr/bin/env bash

file_dir="$(dirname $0)"
fixture_dir="${file_dir}/fixtures"
log_file="${file_dir}/test.log"

echo "Running mock_op.bash with args: \"$@\"" >> $log_file

case "$@" in
  "account get --format json")
    serve_file="${fixture_dir}/account.json"
    ;;
  *)
    echo "No mock specified for command: \"$@\"" >> $log_file
    exit 255
    ;;
esac

echo "Served fixture file $serve_file for command \"$@\"" >> $log_file
cat $serve_file
