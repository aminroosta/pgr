select
  proname name,
  proretset retset,
  pronargs nargs,
  prorettype rettype,
  proargtypes argtypes
from pg_proc
join pg_namespace on pg_namespace.oid = pg_proc.pronamespace
where
  nspname = 'pgr' and
  prokind = 'f' and
  provariadic = 0 -- variadic functions are not supported
;
