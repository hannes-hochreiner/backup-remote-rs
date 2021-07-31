CREATE TABLE vaults (
  id uuid PRIMARY KEY,
  revision uuid NOT NULL,
  creation_date timestamp with time zone NOT NULL,
  inventory_date timestamp with time zone,
  number_of_archives integer NOT NULL,
  size_in_bytes bigint NOT NULL,
  vault_arn varchar(256) UNIQUE NOT NULL,
  vault_name varchar(256) UNIQUE NOT NULL
);

CREATE TABLE archives (
  id uuid PRIMARY KEY,
  revision uuid NOT NULL,
  creation_date timestamp with time zone NOT NULL,
  inventory_date timestamp with time zone,
  vault_id uuid REFERENCES vaults(id),
  size_in_bytes bigint NOT NULL,
  archive_id varchar(256) UNIQUE NOT NULL,
  archive_description varchar(256),
  archive_hash varchar(256) NOT NULL
);

CREATE ROLE updater LOGIN PASSWORD '{{updater_password}}';

GRANT SELECT, INSERT, UPDATE ON vaults TO updater;
GRANT SELECT, INSERT, UPDATE ON archives TO updater;
