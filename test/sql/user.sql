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
