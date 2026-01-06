# Database

Every Stack namespace gets its own Postgres cluster via CloudNativePG. You interact with it the same way you would any Postgres server.

## Connect with kubectl + psql

The demo manifest (`demo-apps/demo.stack.yaml`) creates a database in the `stack-demo` namespace. If you try to connect as `db-owner` directly you may see a peer auth error:

```bash
$ kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -U db-owner -d stack-app
Defaulted container "postgres" out of: postgres, bootstrap-controller (init)
psql: error: connection to server on socket "/controller/run/.s.PGSQL.5432" failed: FATAL:  Peer authentication failed for user "db-owner"
command terminated with exit code 2
```

Instead, open a psql session as the default `postgres` user, then connect to `stack-app`:

```bash
$ kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql
Defaulted container "postgres" out of: postgres, bootstrap-controller (init)
psql (16.1 (Debian 16.1-1.pgdg110+1), server 16.2 (Debian 16.2-1.pgdg110+2))
Type "help" for help.

postgres=# \\l
                                                  List of databases
   Name    |  Owner   | Encoding | Locale Provider | Collate | Ctype | ICU Locale | ICU Rules |   Access privileges
-----------+----------+----------+-----------------+---------+-------+------------+-----------+-----------------------
 postgres  | postgres | UTF8     | libc            | C       | C     |            |           |
 stack-app | db-owner | UTF8     | libc            | C       | C     |            |           |
 template0 | postgres | UTF8     | libc            | C       | C     |            |           | =c/postgres          +
           |          |          |                 |         |       |            |           | postgres=CTc/postgres
 template1 | postgres | UTF8     | libc            | C       | C     |            |           | =c/postgres          +
           |          |          |                 |         |       |            |           | postgres=CTc/postgres
(4 rows)
```

Then connect:

```sql
\\c stack-app
```

## Create a table and insert data

```sql
create table instruments (
  id bigint primary key generated always as identity,
  name text not null
);

insert into instruments (name)
values
  ('violin'),
  ('viola'),
  ('cello');

select * from instruments;
```

## Roles Stack creates for you

Stack bootstraps several roles so different services can connect safely:

- `db-owner` for migrations and admin tasks.
- `application_user` for your app runtime.
- `application_readonly` for read-only access.
- `authenticator` and `anon` for PostgREST.
- `service_role` and `authenticated` for auth-aware workloads.

Connection strings live in the `database-urls` secret and are wired into your app when you set `database_url`, `migrations_database_url`, or `readonly_database_url` in your manifest.

## Quick checks

```bash
kubectl -n stack-demo get pods
kubectl -n stack-demo get secret database-urls -o yaml
```

If you want a long-lived CLI experience, install the CloudNativePG plugin and run:

```bash
kubectl cnpg status stack-db-cluster -n stack-demo
```
