By default, the holon will produce minimal trace logs. These are mainly for general informational purposes.

A holon leverages cargo's features configuration to enable conditional compilation.

To produce more verbose logs during runtime, you must enable the "trace" feature.

Here is a sample command line:

`cargo run --features urbit-api/trace --bin node ralbes-mislec-lodlev-migdev --urbit-port 9030 --node-port 3030`

Notice the --features option with the urbit-api/trace feature enabled. When working with workspaces (such as holon), you must specify features at the packag level: e.g. `<package>/<feature>`

So in the example above, `urbit-api` is the package and `trace` is the feature.
