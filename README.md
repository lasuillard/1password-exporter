# 1password-exporter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/lasuillard/1password-exporter/actions/workflows/ci.yaml/badge.svg)](https://github.com/lasuillard/1password-exporter/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/lasuillard/1password-exporter/graph/badge.svg?token=WTWCSXEMSR)](https://codecov.io/gh/lasuillard/1password-exporter)

1Password personal (not Connect) usage exporter.

## ℹ️ About

This project is hobby-dev Prometheus metrics exporter for 1Password built with Rust. It collects several metrics via `op` CLI.

## 📦 Installation

The exporter provides download option as binary and Docker image.

### ⌨️ Using binary CLI

> [!NOTE]
> You should have 1Password CLI (`op`) installed in your machine and reachable to binary work.

To use binary, download it from releases and run:

```bash
$ OP_EXPORTER_VERSION="0.2.1" wget -qO ./onepassword-exporter "https://github.com/lasuillard/1password-exporter/releases/download/${OP_EXPORTER_VERSION}/onepassword-exporter-x86_64-unknown-linux-musl"
$ chmod +x ./onepassword-exporter
$ ./onepassword-exporter --help
A simple Prometheus exporter for the 1Password

Usage: onepassword-exporter [OPTIONS]

Options:
      --log-level <LOG_LEVEL>
          Log level [default: INFO]
      --host <HOST>
          Host to bind the server to [default: 127.0.0.1]
  -p, --port <PORT>
          Port to bind the server to [default: 9999]
  -m, --metrics <METRICS>...
          Metrics to collect. Only metrics not consuming API rate enabled by default [default: account group user service-account build-info] [possible values: account, build-info, group, service-account, user, document, item, vault]
      --op-path <OP_PATH>
          Path to 1Password CLI binary [default: op]
      --service-account-token <SERVICE_ACCOUNT_TOKEN>
          Service account token to pass to the 1Password CLI
  -h, --help
          Print help
  -V, --version
          Print version
```

### 🐳 Using Docker

The recommended launch option is using Docker. The image already includes OP CLI.

```bash
$ export OP_SERVICE_ACCOUNT_TOKEN="ops_ey..."
$ docker container run -it --rm -e OP_SERVICE_ACCOUNT_TOKEN -p 9999:9999 --init lasuillard/1password-exporter:latest
Unable to find image 'lasuillard/1password-exporter:latest' locally
latest: Pulling from lasuillard/1password-exporter
efc2b5ad9eec: Already exists
d00da3091ee5: Already exists
025e24e306b3: Already exists
ed3747246b1e: Already exists
e912f870d942: Already exists
Digest: sha256:12765cd3062e67fe751953ff48b6a9a5a4f3b8385616c07ef294583e27e2c539
Status: Downloaded newer image for lasuillard/1password-exporter:latest
11:56:16 [INFO] Enabled metrics: [Account, Group, User, ServiceAccount, BuildInfo]
11:56:16 [INFO] Using 1Password CLI: op
11:56:16 [INFO] Listening on http://0.0.0.0:9999
```

Now metrics served at `http://localhost:9999/metrics`. You can find more, such as example Grafana dashboard, at [examples](/examples) directory.

## 📏 Available Metrics

Here is full example of available metrics, with all metrics enabled:

```text
# HELP op_account_current Current 1Password account information.
# TYPE op_account_current gauge
op_account_current{created_at="2023-03-19T05:06:27Z",domain="my",id="??????????????????????????",name="**********",state="ACTIVE",type="FAMILY"} 1
# HELP op_document_count_per_tag Number of documents per tag.
# TYPE op_document_count_per_tag gauge
op_document_count_per_tag{tag="test"} 4
# HELP op_document_count_per_vault Number of documents per vault.
# TYPE op_document_count_per_vault gauge
op_document_count_per_vault{vault="36vhq4xz3r6hnemzadk33evi4a"} 4
# HELP op_document_count_total Total number of documents.
# TYPE op_document_count_total gauge
op_document_count_total 4
# HELP op_document_file_size_per_tag_bytes Size of file in documents per tag, in bytes.
# TYPE op_document_file_size_per_tag_bytes gauge
op_document_file_size_per_tag_bytes{tag="test"} 10494986
# HELP op_document_file_size_per_vault_bytes Size of file in documents per vault, in bytes.
# TYPE op_document_file_size_per_vault_bytes gauge
op_document_file_size_per_vault_bytes{vault="36vhq4xz3r6hnemzadk33evi4a"} 10494986
# HELP op_exporter_buildinfo Build information of this exporter.
# TYPE op_exporter_buildinfo gauge
op_exporter_buildinfo{version="0.4.2"} 1
# HELP op_group_count_total Total number of groups.
# TYPE op_group_count_total gauge
op_group_count_total 4
# HELP op_item_count_per_category Number of items per category.
# TYPE op_item_count_per_category gauge
op_item_count_per_category{category="DOCUMENT"} 1
op_item_count_per_category{category="LOGIN"} 2
op_item_count_per_category{category="SECURE_NOTE"} 1
op_item_count_per_category{category="SSH_KEY"} 1
# HELP op_item_count_per_tag Number of items per tag.
# TYPE op_item_count_per_tag gauge
op_item_count_per_tag{tag="dev"} 1
op_item_count_per_tag{tag="test"} 4
# HELP op_item_count_per_vault Number of items per vault.
# TYPE op_item_count_per_vault gauge
op_item_count_per_vault{vault="36vhq4xz3r6hnemzadk33evi4a"} 5
# HELP op_item_count_total Total number of items.
# TYPE op_item_count_total gauge
op_item_count_total 5
# HELP op_serviceaccount_ratelimit_limit API rate limit.
# TYPE op_serviceaccount_ratelimit_limit gauge
op_serviceaccount_ratelimit_limit{action="read",type="token"} 1000
op_serviceaccount_ratelimit_limit{action="read_write",type="account"} 1000
op_serviceaccount_ratelimit_limit{action="write",type="token"} 100
# HELP op_serviceaccount_ratelimit_remaining API rate limit remaining.
# TYPE op_serviceaccount_ratelimit_remaining gauge
op_serviceaccount_ratelimit_remaining{action="read",type="token"} 999
op_serviceaccount_ratelimit_remaining{action="read_write",type="account"} 999
op_serviceaccount_ratelimit_remaining{action="write",type="token"} 100
# HELP op_serviceaccount_ratelimit_reset_seconds API rate limit remaining.
# TYPE op_serviceaccount_ratelimit_reset_seconds gauge
op_serviceaccount_ratelimit_reset_seconds{action="read",type="token"} 308
op_serviceaccount_ratelimit_reset_seconds{action="read_write",type="account"} 83108
op_serviceaccount_ratelimit_reset_seconds{action="write",type="token"} 0
# HELP op_serviceaccount_ratelimit_used API rate limit used.
# TYPE op_serviceaccount_ratelimit_used gauge
op_serviceaccount_ratelimit_used{action="read",type="token"} 1
op_serviceaccount_ratelimit_used{action="read_write",type="account"} 1
op_serviceaccount_ratelimit_used{action="write",type="token"} 0
# HELP op_serviceaccount_whoami Current service account information.
# TYPE op_serviceaccount_whoami gauge
op_serviceaccount_whoami{account_uuid="++++++++++++++++++++++++++",url="https://my.1password.com",user_type="SERVICE_ACCOUNT",user_uuid="!!!!!!!!!!!!!!!!!!!!!!!!!!"} 1
# HELP op_user_count_total Total number of users.
# TYPE op_user_count_total gauge
op_user_count_total 1
# HELP op_vault_count_total Total number of vaults.
# TYPE op_vault_count_total gauge
op_vault_count_total 1
```

## ⚠️ Limitations

Due to how the OP CLI and the exporter works, there are several known limitations:

- Exporter can collect metrics from vaults SA has read access.
- Any newly created vaults won't be tracked because 1Password does not support automatic access grant for SA to newly created vaults. If you added new vault, you should create new SA and update the SA token.
- Some vaults (e.g. Private) is impossible to share with SA, therefore metrics for those vaults cannot be collected.

## 💖 Contributing

Please submit issues or pull requests for questions, bugs, or requests for new features.

## 📜 License

This project is licensed under the terms of the MIT license.
