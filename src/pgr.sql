begin;
drop schema if exists pgr cascade;
create schema pgr;
set search_path to pgr, public;

create function _pgr_to_text_oid(oid oid) returns text as $$
  select
    case
      when pg_namespace.nspname::text = 'pg_catalog' then pg_type.typname::text
      else pg_namespace.nspname::text || '.' || pg_type.typname::text
    end
  from pg_type
  join pg_namespace on pg_type.typnamespace = pg_namespace.oid
  where pg_type.oid = $1;
$$ language sql;

create function _pgr_functions(schema_name text)
returns table (
  name text,
  retset boolean,
  rettype text,
  argtypes text[],
  argnames text[],
  argmodes text[]
) as $$
begin
  return query
    select
      proname::text name,
      proretset retset,
      pgr._pgr_to_text_oid(prorettype::regtype) rettype,
      array(
        select pgr._pgr_to_text_oid(
          unnest(
            coalesce(proallargtypes, proargtypes)
          )::regtype
        )
      ) as argtypes,
      coalesce(proargnames, '{}') argnames,
      array(
        select
          case argmode
            when 'i' then 'in'
            when 'o' then 'out'
            when 'b' then 'inout'
            when 'v' then 'variadic'
            when 't' then 'table'
            else null
          end
        from unnest(coalesce(proargmodes, '{}')) argmode
      ) as argmodes
    from pg_proc
    join pg_namespace on pg_namespace.oid = pg_proc.pronamespace
    where
      nspname = schema_name and
      prokind = 'f' and
      provariadic = 0 and
      proname not like '_pgr_%'
    ;
end;
$$ language plpgsql;

PLACEHOLDER

commit;
