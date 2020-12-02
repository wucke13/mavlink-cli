# About

A tool to interact with MAVLink compatible vehicles.

Currently the majority of the features is aimed at configuration management.
However, it is planned to extend the scope
of this tool to other MAVLink related tasks as well, hence the name.

![`configure` mode](https://user-images.githubusercontent.com/20400405/93691947-ca363f00-faec-11ea-8727-260a467ea056.png)

[more screenshots](https://github.com/wucke13/mavlink-cli/issues/1)

# Why

I always wanted a neat yet feature rich CLI tool for mavlink. 

# Usage

from the `mavlink-cli --help` output:

```
USAGE:
    mavlink-cli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -c, --connection <mavlink-connection>
            MAVLink connection string. (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)
            [default: udpbcast:0.0.0.0:14551]

SUBCOMMANDS:
    configure
            Interactive configuration management

            Starts a fuzzy finder which allows to search through the MAVLink parameters available on the connected
            vehicle. Select one ([Return]) or multiple ([Tabulator]) parameters which you would like to inspect. You can
            modify them, including sanity checking if metainformation is avaibable on the parameter.
    help
            Prints this message or the help of the given subcommand(s)

    info
            Browse all parameters with available metainformation

            Starts a fuzzy finder which allow to search through the MAVLink paramters for which metainformation is
            available. Select one ([Return]) or multiple ([Tabulator]) parameters which you would like to inspect. The
            avaibable metainformation for each parameter is printed to stdout.
    pull
            Pull configuration from the vehicle to a file

    push
            Push configuration from a file to the vehicle
```

# Planned features

+ [ ] PX4 support
+ [ ] no waiting for all parameters to arrive in `configure` mode
+ [x] fuzzy search through descriptions as well (see [this issue](https://github.com/lotabout/skim/issues/344)
+ [ ] better Error reporting
+ [ ] report flag, which enable a detailed report about which parameters where changed on program termination
+ [ ] motor test capability
+ [ ] live monitiroing of attitude, battery telemetry & more
+ [ ] in `configure` mode show current value in preview
+ [ ] Implement local parameter repo

# Todo

+ [ ] evaluate [`cursive`](https://lib.rs/crates/cursive), it might be a good
  replacement for skim + dialoguer
+ [ ] document everything public
+ [ ] refined support for the mavlink parameter protocol
+ [ ] refine user interaction
+ [ ] retain last search in `configure` mode
+ [x] implement current value adoption for Bitmask
+ [ ] sending heartbeat ourselves
+ [ ] detecting missing communication
+ [ ] better errorhandling in the `mavlink_stub` module
+ [ ] PATH like mechanism for parameter definition files
+ [ ] Fix super slow `push`
+ [ ] Enable build on windows

# Disclaimer

I develop this project in my free time out of personal interest in the topic.
You're welcome to leave feedback in the issues. However, as my time for working
on this is limited, I might be slow to respond.
