-- create a composite type for user
create type user_t AS (
  user_id integer,
  name text
);

-- get a user by id
create function "get /user/:id"(id integer) returns user_t as $$
  select * from users where user_id = id;
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
create function "get /user/:id?name&age"(
  id integer,
  name text,
  age integer
) returns user_t as $$
  select * from users where user_id = id and name = name;
$$ language sql;

-- create a hook function to parse the cookie header
create function "hook in cookie"(headers text[])
returns text as $$
  select v
  from unnest(headers) with ordinality hdr(k, v)
  where k = 'Cookie';
$$ language sql;

create function "get /user/:id"(id integer, cookie text) returns user_t as $$
  select * from users where user_id = id and cookie = cookie;
$$ language sql;

create function "post /user"(body user_t, out cookie text, out response user_t) as $$
begin
  insert into users values (body.*) returning * into  response.user_id, response.name;
  cookie := 'user_id=' || body.user_id;
end
$$ language plpgsql;

create function "hook inout cookie"(value text, inout headers text[]) as $$
begin
  headers := headers || array[['Set-Cookie', value]];
end
$$ language plpgsql;

select * from pgr."hook inout cookie"('user_id=6', array['host', 'localhost']);

delete from users;
select
  to_json(response) response,
  pgr."hook inout cookie"(cookie, array['host', 'localhost']) headers,
  200 status
from pgr."post /user"(
  jsonb_populate_record(
    null::pgr.user_t, '{"user_id": 6, "name": "Alice"}'::jsonb
  )
)
;