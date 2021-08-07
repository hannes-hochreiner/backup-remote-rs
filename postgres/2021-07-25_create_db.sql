CREATE TABLE vaults (
  creation_date timestamp with time zone NOT NULL,
  inventory_date timestamp with time zone,
  number_of_archives bigint NOT NULL,
  size_in_bytes bigint NOT NULL,
  vault_arn varchar(256) PRIMARY KEY,
  vault_name varchar(256) UNIQUE NOT NULL
);

CREATE TABLE jobs (
  job_id varchar(256) PRIMARY KEY,
  action varchar(256) NOT NULL,
  archive_id varchar(256) REFERENCES archives(archive_id),
  archive_tree_hash varchar(256),
  archive_size_in_bytes bigint,
  completion_date timestamp with time zone,
  creation_date timestamp with time zone NOT NULL,
  inventory_size_in_bytes bigint,
  job_description varchar(256),
  tree_hash varchar(256),
  status_code varchar(256) NOT NULL,
  status_message varchar(512),
  vault_arn varchar(256) REFERENCES vaults(vault_arn)
);

CREATE TABLE archives (
  creation_date timestamp with time zone NOT NULL,
  inventory_date timestamp with time zone,
  vault_id varchar(256) REFERENCES vaults(vault_arn),
  size_in_bytes bigint NOT NULL,
  archive_id varchar(256) PRIMARY KEY,
  archive_description varchar(256),
  archive_hash varchar(256) NOT NULL
);

CREATE ROLE updater LOGIN PASSWORD '{{updater_password}}';

GRANT SELECT, INSERT, UPDATE ON vaults TO updater;
GRANT SELECT, INSERT, UPDATE ON archives TO updater;
