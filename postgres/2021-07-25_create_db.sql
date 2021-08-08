CREATE TABLE vaults (
  creation_date timestamp with time zone NOT NULL,
  last_inventory_date timestamp with time zone,
  number_of_archives bigint NOT NULL,
  size_in_bytes bigint NOT NULL,
  vault_arn varchar(256) PRIMARY KEY,
  vault_name varchar(256) UNIQUE NOT NULL
);

CREATE TABLE vaults_status (
  vault_arn varchar(256) PRIMARY KEY REFERENCES vaults(vault_arn),
  active boolean NOT NULL
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

CREATE TABLE jobs_status (
  job_id varchar(256) PRIMARY KEY REFERENCES jobs(job_id),
  active boolean NOT NULL
);

CREATE TABLE jobs_workers (
  job_id varchar(256) PRIMARY KEY REFERENCES jobs(job_id),
  vault_arn varchar(256) REFERENCES vaults(vault_arn),
  completed boolean NOT NULL,
  pid oid NOT NULL
);

CREATE ROLE updater LOGIN PASSWORD '{{updater_password}}';

GRANT SELECT, INSERT, UPDATE ON vaults TO updater;
GRANT SELECT, INSERT, UPDATE ON vaults_status TO updater;
GRANT SELECT, INSERT, UPDATE ON archives TO updater;
GRANT SELECT, INSERT, UPDATE ON jobs TO updater;
GRANT SELECT, INSERT, UPDATE ON jobs_status TO updater;
GRANT SELECT, INSERT, UPDATE ON jobs_workers TO updater;

CREATE ROLE worker LOGIN PASSWORD '{{worker_password}}';

GRANT SELECT, INSERT, UPDATE ON jobs_workers TO worker;

CREATE ROLE api LOGIN PASSWORD '{{api_password}}';

GRANT SELECT ON vaults TO api;
GRANT SELECT ON vaults_status TO api;
GRANT SELECT ON archives TO api;
GRANT SELECT, INSERT, UPDATE ON jobs TO api;
GRANT SELECT, INSERT, UPDATE ON jobs_status TO api;
GRANT SELECT ON jobs_workers TO api;
