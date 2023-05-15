## Holium node CLI

```txt
> hol
Hol: a node for P2P applications.
https://holium.com
Version 0.1

Usage: hol boot --fake -p 9030 zod
       hol nodes
       hol node zod start

Commands:
  boot        Boots a node with the identity provided.
  nodes       List all running nodes
  node        Attach to instances for logs, metrics, and running commands
  version     Prints the current version
```

### `boot` command

```text
> hol boot

Usage: hol boot [OPTIONS] zod

Options:
  -F, --fake        Boots a fake ship
  -p, --urbit-port  http-port for Urbit instance
  -P, --node-port   wrapper node port (proxies to urbit-port)
  -G, --key         urbit id keyfile in string form
```

### `nodes` command

```text
> hol nodes
+----------------+----------+----------+----------+----------+
| server-id      | status   | pid      | type     | port     |
+----------------+----------+----------+----------+----------+ 
| lomder-librun  | running  | 42561    | live     | 3030     |
+----------------+----------+----------+----------+----------+
| zod            | stoppped | 43215    | fake     | 3031     |
+----------------+----------+----------+----------+----------+
| bus            | stopped  | 48901    | fake     | 3032     |
+----------------+----------+----------+----------+----------+
```

### `node` command

```txt
> hol node
Usage: hol node [OPTIONS] [server-id] logs

Options:
  -v, --verbose     Prints verbose logs

Commands:
  start       Starts the Urbit instance
  stop        Stops the Urbit instance
  clean       Runs a cleanup script (pack, meld, chop, etc.)
  info        Prints the current vere and urbit version
  logs        Prints logs (--attach to connect to stdout)
  command     Run a command against the instance.
  upgrade     Checks for updates and applies them
  apps        Returns a list of all apps (agents) running with all metadata
  app         A subcommand for managing apps
```

### `version` command

```text
> hol version
0.1
```