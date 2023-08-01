# Getting Started

Guide that should be focused on how to get started as a developer looking to leverage holon features; e.g. websockets. For general installation, setup, and configuration please refer to the general README.md.

## Tips

- use the `trace` library for all terminal/cli logging

  - contains the following tracing macros which adds color coding and other contextual information to all printouts:

    - `trace_info`, `trace_info_ln`
    - `trace_warn`, `trace_warn_ln`
    - `trace_err` , `trace_err_ln`
    - `trace_good`, `trace_good_ln`

- For colored json output, always use the `trace_json` and `trace_json_ln` macros.

## Javascript
