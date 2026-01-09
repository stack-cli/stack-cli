# Realtime

Stack can run Supabase Realtime for your namespace so you can subscribe to database changes over WebSockets.

This page continues the demo flow from the [Database](../database/) and [REST](../rest/) guides.

## Quick Start

Open a psql session as the default user, then connect to `stack-app`:

```bash
kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -d stack-app
```

### Create a table in your Stack database

```sql
-- Create a table called "todos"
-- with a column to store tasks.
create table todos (
  id serial primary key,
  task text
);
```

### Allow anonymous access

```sql
-- Turn on security
alter table "todos"
enable row level security;

-- Allow anonymous access
create policy "Allow anonymous access"
on todos
for select
to anon
using (true);
```

### Enable Postgres Replication

```sql
alter publication supabase_realtime
add table todos;
```

### Install the client

Realtime uses websockets so the best way to test it is to install the Supabase JavaScript client.

```sh
npm install @supabase/supabase-js dotenv
```

### Create the client

Create a `.env` from Stack secrets, then load it from the script. The channel name can be any string except `realtime`.

```bash
stack secrets --manifest demo.stack.yaml > .env
```

```bash
cat > realtime-listen.mjs <<'EOF'
import dotenv from 'dotenv'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import { createClient } from '@supabase/supabase-js'

const __dirname = dirname(fileURLToPath(import.meta.url))
dotenv.config({ path: join(__dirname, '.env'), override: true })

const supabaseUrl = 'http://localhost:30010'
const supabaseKey = process.env.ANON_JWT

if (!supabaseKey) {
  console.error('Missing ANON_JWT. Export it before running this script.')
  process.exit(1)
}

const supabase = createClient(supabaseUrl, supabaseKey)

supabase
  .channel('schema-db-changes')
  .on(
    'postgres_changes',
    {
      event: '*',
      schema: 'public',
      table: 'todos',
    },
    (payload) => console.log(payload)
  )
  .subscribe((status, err) => {
    if (err) {
      console.error('Realtime error:', err)
    } else {
      console.log('Realtime status:', status)
    }
  })
EOF
```

Run it:

```bash
node realtime-listen.mjs
```

### Insert dummy data

Back in `psql`, insert a row to trigger the change event:

```sql
insert into todos (task)
values
  ('Change!');
```

## Technical notes

- Realtime runs as a separate deployment in your namespace.
- It uses the shared `jwt-auth` secret for API JWT validation.
- The nginx gateway exposes it under `/realtime/v1`.
