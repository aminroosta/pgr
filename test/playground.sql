begin;

select
  proname name,
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
  nspname = 'pgr' and
  prokind = 'f' and
  provariadic = 0 -- variadic functions are not supported
;

rollback;
