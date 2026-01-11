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

-- Grant table access for anon + service role
grant usage on schema public to anon, authenticated, service_role;
grant select on table public.todos to anon, authenticated;
grant all on table public.todos to service_role;
```

### Enable Postgres Replication

```sql
alter publication supabase_realtime
add table todos;
```

### Install the client

Realtime uses WebSockets. You can test it with a minimal Node script.

```sh
npm install ws dotenv
```

### Create the client

Create a `.env` from Stack secrets, then create `test.mjs`:

```bash
stack secrets --manifest demo.stack.yaml > .env
```

```bash
cat > test.mjs <<'EOF'
import WebSocket from "ws";
import dotenv from "dotenv";

dotenv.config();

const jwt = process.env.ANON_JWT;
if (!jwt) {
  console.error("Set ANON_JWT before running this script.");
  process.exit(1);
}

const url = `ws://host.docker.internal:30010/realtime/v1/websocket?apikey=${jwt}`;
const ws = new WebSocket(url, "realtime-v1", {
  headers: {
    apikey: jwt,
    Authorization: `Bearer ${jwt}`,
  },
});

ws.on("open", () => {
  console.log("open");
  const join = {
    topic: "realtime:public:todos",
    event: "phx_join",
    payload: {
      config: {
        broadcast: { ack: false },
        postgres_changes: [
          { event: "*", schema: "public", table: "todos" },
        ],
      },
    },
    ref: "1",
  };
  ws.send(JSON.stringify(join));
});
ws.on("message", (data) => console.log(data.toString()));
ws.on("error", (err) => console.error(err));
ws.on("close", (code, reason) => console.log("close", code, reason.toString()));
EOF
```

Run it:

```bash
node test.mjs
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
