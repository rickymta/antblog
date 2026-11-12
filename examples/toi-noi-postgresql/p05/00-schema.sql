-- Projection vận hành thu nhỏ, không thay schema P01–P04.
CREATE ROLE tn_owner NOLOGIN;
CREATE ROLE tn_app LOGIN;
CREATE ROLE tn_report NOLOGIN;
CREATE ROLE tn_monitor NOLOGIN;
GRANT pg_monitor TO tn_monitor;
CREATE SCHEMA ops AUTHORIZATION tn_owner;
SET ROLE tn_owner;
CREATE TABLE ops.trips (
  id bigint PRIMARY KEY,
  tenant_id integer NOT NULL,
  fare bigint NOT NULL CHECK (fare >= 0),
  status text NOT NULL CHECK (status IN ('completed', 'cancelled'))
);
INSERT INTO ops.trips
SELECT n, CASE WHEN n <= 50 THEN 1 ELSE 2 END,
       30000, 'completed' FROM generate_series(1,100) n;
GRANT USAGE ON SCHEMA ops TO tn_app, tn_report;
GRANT SELECT, INSERT, UPDATE ON ops.trips TO tn_app;
GRANT SELECT ON ops.trips TO tn_report;
ALTER DEFAULT PRIVILEGES IN SCHEMA ops GRANT SELECT ON TABLES TO tn_report;
RESET ROLE;
