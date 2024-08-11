#!/usr/bin/env bash

file_dir="$(dirname $0)"
fixture_dir="${file_dir}/fixtures"
log_file="${file_dir}/test.log"

echo "Running mock_op.bash with args: \"$@\"" >> $log_file

case "$@" in
  "account get --format json")
    serve_file="${fixture_dir}/account.json"
    ;;
  "document list --format json --include-archive")
    serve_file="${fixture_dir}/document.json"
    ;;
  "group list --format json")
    serve_file="${fixture_dir}/group.json"
    ;;
  "item list --format json --include-archive")
    serve_file="${fixture_dir}/item.json"
    ;;
  "service-account ratelimit --format json")
    serve_file="${fixture_dir}/ratelimit.json"
    ;;
  "whoami --format json")
    serve_file="${fixture_dir}/whoami.json"
    ;;
  "user list --format json")
    serve_file="${fixture_dir}/user.json"
    ;;
  "vault list --format json")
    serve_file="${fixture_dir}/vault.json"
    ;;
  *)
    echo "No mock specified for command: \"$@\"" >> $log_file
    exit 255
    ;;
esac

echo "Served fixture file $serve_file for command \"$@\"" >> $log_file
cat $serve_file
