select
  -- TODO: support default arguments
  -- proargdefaults argdefaults,
  -- prosrc src,
  proname name,
  proretset retset,
  pronargs nargs,
  prorettype rettype,
  coalesce(proallargtypes, proargtypes) as argtypes,
  coalesce(proargmodes, '{}') as argmodes,
  coalesce(proargnames, '{}') argnames
from pg_proc
join pg_namespace on pg_namespace.oid = pg_proc.pronamespace
where
  nspname = 'pgr' and
  prokind = 'f' and
  provariadic = 0 -- variadic functions are not supported
;

