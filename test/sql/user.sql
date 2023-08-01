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

-- get a user by id and name
create function "get /user?id&name"(id integer, name text) returns user_t as $$
  select * from users where user_id = id and name = name;
$$ language sql;

-- create a hook function to parse the cookie header
create function "hook in cookie"(headers text[][]) returns text as $$
  select value from unnest(headers) where key = 'Cookie';
$$ language sql;

-- create a hook function to set the cookie header
create function "hook out cookie"(value text, out headers text[][]) as $$
  headers := array[['Set-Cookie', value]];
$$ language sql;

-- get a user by id and cookie
create function "get /user/:id"(id integer, cookie text) returns user_t as $$
  select * from users where user_id = id and cookie = cookie;
$$ language sql;

-- create a new user and set a cookie
create function "post /user"(body user_t, out cookie text) returns user_t as $$
  insert into users values (body.*) returning *;
  cookie := 'user_id=' || body.user_id;
$$ language sql;
