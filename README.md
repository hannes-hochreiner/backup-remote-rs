# backup-remote-rs
A tool for managing remote backups in AWS Glacier.

# Note
This tool uses the AWS Glacier API.
The Rust SDK does not support Glacier yet (July 2021).

# Prerequisites
The tool needs some basic information about the Glacier account and a system user with access to the account.
The information is provided as environment variables.

| Name | Description |
| ---: | --- |
| AWS_REGION | region, where the vault is located (e.g. "eu-central-1") |
| AWS_SECRET_KEY | secret access key obtained when creating the user |
| AWS_KEY_ID | key id obtained when creating the user |
| DB_CONNECTION | database connection (e.g. "postgresql://&lt;updater db user&gt;:&lt;updater password&gt;@&lt;host&gt;:5432/rss_json") |
| RUST_LOG | log level (i.e. error, warn, info, debug, trace) |

# Development

## Setup

An Ansible script automating this process can be found in the `ansible` folder.
The scripts expects the variables listed in the table below, which must be provided in a file names `vars.yml` in the Ansible folder.

| variable name | description |
| ------------- | ----------- |
| postgres_password | db master password |
| updater_password | password for the updater user |

If the passwords are encrypted with Ansible vault, the ansible script can be run with the following command:

```bash
ansible-playbook --ask-vault-pass ansible/db_create_pb.yml
```
