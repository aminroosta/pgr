# pgr: postgres relief

pgr is a project that allows you to run SQL scripts as REST and WebSocket
endpoints.

## Features

- You can define SQL functions that map to REST endpoints using a simple naming convention.
- You can use path variables, query string parameters, and request body as function arguments.
- You can use hook functions to handle common tasks such as parsing or setting cookies, JWT authentication, WebSocket sessions, etc.
- You can use a config file to specify the database connection details, the JWT secret, and other options.

## Usage

- Create a config file named `pgr.conf` in your project directory. Here is an example:

```conf
[database]
host = localhost
port = 5432
user = postgres
password = secret
dbname = postgres

[jwt]
secret = supersecret
```

- Create a directory named `sql` in your project directory and put your SQL
  files there. Each file should contain one or more function definitions that
  map to REST endpoints or WebSocket events. Here is an example:

```sql
-- user.sql

-- create a composite type for user
create type user_t AS (
  user_id integer,
  name text
);

-- get a user by id
create function "get /user/:id"(id integer) returns user_t as $$
  select * from users where user_id = id;
$$ language sql;

-- create a new user
create function "post /user"(body user_t) returns user_t as $$
  insert into users values (body.*) returning *;
$$ language sql;

-- update a user by id
create function "put /user/:id"(id integer, body user_t) returns user_t as $$
  update users set name = body.name where user_id = id returning *;
$$ language sql;

-- delete a user by id
create function "delete /user/:id"(id integer) returns void as $$
  delete from users where user_id = id;
$$ language sql;
```

- Run `pgr` in your project directory. It will scan the `sql` directory and
  create a schema for each file. It will also start a web server that listens
  for requests on port 8080 (by default).

- You can now send requests to the endpoints defined by your SQL functions. For example:

```sh
curl http://localhost:8080/user/1 # get the user with id 1
curl -X POST http://localhost:8080/user -d '{"user_id":2,"name":"Alice"}' # create a new user with id 2 and name Alice
curl -X PUT http://localhost:8080/user/2 -d '{"name":"Bob"}' # update the name of the user with id 2 to Bob
curl -X DELETE http://localhost:8080/user/2 # delete the user with id 2
```

## Query string parameters

You can use query string parameters to pass additional arguments to your
endpoint functions. To use query string parameters, you need to name your
function arguments with the same name as the query string keys, without the `:`
prefix. For example:

```sql
-- get a user by id and name
create function "get /user?id&name"(id integer, name text) returns user_t as $$
  select * from users where user_id = id and name = name;
$$ language sql;
```

You can then send requests to the endpoint with query string parameters. For example:

```sh
curl http://localhost:8080/user?id=1&name=Alice # get the user with id 1 and name Alice
```

## Hooks and cookies

Hooks are special functions that can be used to handle common tasks such as
parsing or setting cookies, JWT authentication, WebSocket events, etc. You can
define hook functions in your SQL files using the `hook` keyword followed by
the direction (`in` or `out`) and the argument name. For example:

```sql
-- create a hook function to parse the cookie header
create function "hook in cookie"(headers text[][]) returns text as $$
  select value from unnest(headers) where key = 'Cookie';
$$ language sql;

-- create a hook function to set the cookie header
create function "hook out cookie"(value text, out headers text[][]) as $$
  headers := array[['Set-Cookie', value]];
$$ language sql;
```

You can then use the hook functions as arguments in your endpoint functions. For example:

```sql
-- get a user by id and cookie
create function "get /user/:id"(id integer, cookie text) returns user_t as $$
  select * from users where user_id = id and cookie = cookie;
$$ language sql;

-- create a new user and set a cookie
create function "post /user"(body user_t, out cookie text) returns user_t as $$
  insert into users values (body.*) returning *;
  cookie := 'user_id=' || body.user_id;
$$ language sql;
```
## Built-in hooks

pgr provides some built-in hooks that you can use in your endpoint functions or WebSocket handlers. These hooks are:

- `config`: the config file as a JSON object. You can use this hook to access
  the configuration options in your endpoint functions or WebSocket handlers.
```sql
-- create a function that returns the database name from the config file
create function "get /dbname"(config jsonb) returns text as $$
  -- return the dbname value
  return config->'database'->>'dbname';
$$ language sql;
```
- `headers`: an array of key-value pairs that represents the HTTP headers of
  the request or the response. You can use this hook as an `in` or `out`
  argument to get or set the headers.

- `status`: an integer that represents the HTTP status code of the response.
  You can use this hook as an `out` argument to set the status code explicitly.

- `session`: a JSON object that represents the WebSocket session data. You can
  use this hook to store or retrieve any information related to the WebSocket
  connection, such as user id, preferences, state, etc.

```sql
create function "ws /on/connected"(headers text[][], out session jsonb, out response jsonb) as $$
  -- get the user id from the headers
  declare user_id text := (select value from unnest(headers) where key = 'X-User-Id');
  -- set the session user id
  session := jsonb_build_object('user_id', user_id);
  -- send a welcome message
  response := jsonb_build_object('type', 'welcome', 'user_id', user_id);
$$ language sql;
```

The session hook is only available for WebSocket handlers, because it is tied
to the WebSocket connection. Unlike REST endpoints, which are stateless and
handle each request independently, WebSocket handlers are stateful and maintain
a persistent connection with the client.

```sql
create function "ws /on/type=chat"(body jsonb, session jsonb) returns jsonb as $$
  -- get the user id from the session
  declare user_id text := session->>'user_id';
  -- get the message from the body
  declare message text := body->>'message';
  -- return a chat message with the user id and message
  return jsonb_build_object('type', 'chat', 'user_id', user_id, 'message', message);
$$ language sql;
```

## WebSocket handlers

WebSocket handlers are functions that handle WebSocket events such as
connection, disconnection, and message. You can define WebSocket handlers in
your SQL files using the `ws` keyword followed by the event name and the path.
For example:

```sql
-- handle WebSocket connection
create function "ws /on/connected"(headers text[][], out session jsonb, out response jsonb) as $$
  session := jsonb_build_object('user_id', null);
  response := jsonb_build_object('type', 'welcome');
$$ language sql;

-- handle WebSocket disconnection
create function "ws /on/disconnected"(session jsonb) returns void as $$
  -- do some cleanup
$$ language sql;

-- handle WebSocket message with type auth
create function "ws /on/type=auth"(body jsonb, inout session jsonb, out response jsonb) as $$
  -- validate the body and set the session user_id
  session := session || body;
  response := jsonb_build_object('type', 'auth', 'status', 'ok');
$$ language sql;

-- handle WebSocket message with type ping
create function "ws /on/type=ping"(body jsonb, session jsonb) returns jsonb as $$
  return jsonb_build_object('type', 'pong');
$$ language sql;
```
## Listen and notify

You can use the PostgreSQL `listen` and `notify` commands to send messages from
the database to the WebSocket clients. To use this feature, you need to do the following:

- In your WebSocket handler, use the `listen` command to subscribe to a channel with the same name as the path. For example:

```sql
-- handle WebSocket connection
create function "ws /on/connected"(headers text[][], out session jsonb, out response jsonb) as $$
  session := jsonb_build_object('user_id', null);
  response := jsonb_build_object('type', 'welcome');
  listen '/on/connected'; -- subscribe to the /on/connected channel
$$ language sql;
```

- In any other function, use the `notify` command to send a message to the channel. For example:

```sql
-- create a new user and notify the /on/connected channel
create function "post /user"(body user_t) returns user_t as $$
  insert into users values (body.*) returning *;
  notify '/on/connected', jsonb_build_object('type', 'new_user', 'user', body.*)::text; -- send a message to the /on/connected channel
$$ language sql;
```

## SQL files

pgr looks for SQL files in the directory specified by the `sql_dir` option in
your config file. The default value is `sql`. Each SQL file should contain one
or more function definitions that map to REST endpoints or WebSocket events.
You can use any valid SQL syntax in your function definitions, as long as they
follow the naming convention and the argument types.

pgr wraps all the SQL files in a single transaction and runs them in the
database. All the files end up in a single schema named `pgr`. For example, if
you have two files named `user.sql` and `post.sql`, they will be executed as:

```sql
begin;
drop schema if exists pgr cascade;
create schema pgr;
set search_path to pgr;
-- user.sql file content
-- post.sql file content
commit;
```

You should not define tables in your SQL files, as they are meant for function
definitions only. You should create your tables in a separate schema and
reference them with qualified names when you query them from your functions.
For example, if you have a table named `users` in the `public` schema, you need
to use `public.users` to access it. For example:

```sql
-- user.sql

-- get a user by id
create function "get /user/:id"(id integer) returns public.users as $$
  select * from public.users where user_id = id; -- need to use public.users here
$$ language sql;
```

