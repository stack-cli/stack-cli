# Storage

Stack ships a Supabase Storage API deployment so your app can serve and store files without wiring S3 by hand. By default the controller deploys MinIO in your namespace, generates credentials, and injects them plus a JWT secret into the storage pod.

This page continues the demo flow from the [Database](../database/) and [REST](../rest/) guides.

## Quick local test (demo manifest)

With the demo manifest, you can hit the Storage API via the nginx gateway at `/storage` (see `stack status` for the JWTs to use).

First grab the service role JWT from `stack status`:

```bash
stack status --manifest demo-stack-app.yaml
```

Then create a bucket:

```bash
curl --location --request POST 'http://host.docker.internal:30010/storage/v1/bucket' \
  --header 'Authorization: Bearer <SERVICE_ROLE_JWT>' \
  --header 'Content-Type: application/json' \
  --data-raw '{"name": "avatars"}'
```

Verify in Postgres:

```bash
kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -U db-owner -d stack-app \
  -c 'select * from storage.buckets;'
```

### Upload a file

```bash
echo "hello storage" > hello.txt
curl -X POST 'http://host.docker.internal:30010/storage/v1/object/avatars/hello.txt' \
  -H 'Authorization: Bearer <SERVICE_ROLE_JWT>' \
  -H 'Content-Type: text/plain' \
  --data-binary @hello.txt
```

Then check the DB:

```bash
kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -U db-owner -d stack-app \
  -c 'select id, name, bucket_id, version from storage.objects;'
```

## What the controller creates

- A `storage` Deployment using `supabase/storage-api` on port `5000`.
- A `storage-s3` Secret with generated AWS-compat credentials and endpoint/bucket settings.
- A `jwt-auth` Secret with a random JWT secret (shared with REST and Realtime).
- A MinIO Deployment/Service when you have not provided your own S3 secret.

### Using the default MinIO

If you omit `s3_secret_name`, the controller:

- Creates `storage-s3` with defaults (bucket `supa-storage-bucket`, endpoint `http://minio:9000`, region `us-east-1`, path-style forced).
- Generates random access keys and S3 protocol keys.
- Deploys MinIO with those credentials.

### Using an external S3/MinIO

Create a secret (any name) in your app namespace with these keys:

- `STORAGE_S3_BUCKET`
- `STORAGE_S3_ENDPOINT` (e.g. `https://s3.amazonaws.com` or `http://minio.example:9000`)
- `STORAGE_S3_REGION`
- `STORAGE_S3_FORCE_PATH_STYLE` (`"true"` or `"false"`)
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `S3_PROTOCOL_ACCESS_KEY_ID`
- `S3_PROTOCOL_ACCESS_KEY_SECRET`

Then set `s3_secret_name` to that secret and `install_minio: false` if you don’t want the bundled MinIO.

## Database wiring

- Storage uses the `migrations-url` from the `database-urls` secret so it can run migrations.
- JWT auth uses the shared `jwt-auth` secret and HS256.
- Supabase Storage keeps metadata in your app’s Postgres database under the `storage` schema (tables like `buckets`, `objects`, and policies). Plan migrations/backups accordingly—metadata and object store must stay in sync.

## Customising with the CRD

Add a storage block to your `StackApp`:

```yaml
spec:
  components:
    storage:
      # Optional: point at your own S3 secret and skip MinIO
      s3_secret_name: my-s3-secret
      install_minio: false
```
