# backup-remote-rs
A tool for managing remote backups in AWS Glacier.

# Prerequisites
The tool needs some basic information about the Glacier account and a system user with access to the account.
The information is provided as environment variables.

| Name | Description |
| ---: | --- |
| AWS_REGION | region, where the vault is located (e.g. "eu-central-1") |
| AWS_SECRET_KEY | secret access key obtained when creating the user |
| AWS_KEY_ID | key id obtained when creating the user |
