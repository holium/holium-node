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
  install     Installs the Urbit binary
  boot        Boots a node with the identity provided.
  network     Configure network topology & discovery mechanism
  instance    Attach to instances for logs, metrics, and running commands
  version     Prints the current version
```

### `node` command

```txt
> hol node
Usage: hol node [OPTIONS] [server-id] logs

Options:
  -v, --verbose     Prints verbose logs

Commands:
  install     Installs the Urbit binary
  boot        Boots an identity and exits.
  start       Starts the instance for the ID registered with the node
  stop        Stops the Urbit instance
  clean       Runs a cleanup script (pack, meld, chop, etc.)
  info        Prints the current vere and urbit version
  logs        Prints logs (--attach to connect to stdout)
  command     Run a command against the instance.
  upgrade     Checks for updates and applies them
  apps        Returns a list of all apps (agents) running with all metadata (docket)
  app         A subcommand for managing apps
  version     Prints the current version of the Holium node
```

### `version` command

```text
> hol version
0.1
```
