# ogn-client-rs

A client for the [Open Glider Network](http://wiki.glidernet.org/) written in Rust.


## Get the binary

There are two ways: you can just checkout/download this repository, [install Rust](https://www.rust-lang.org/tools/install), run ```cargo build --release``` and then take the binary from subdirectory ```target/release```,
or you go to the [releases page](https://github.com/Meisterschueler/ogn-client-rs/releases) and download the
compiled binary for the desired platform.

## Example Usage
### Get the raw stream

Just start the client. It will connect to the OGN data stream and print it out. Every message received is prefixed by a timestamp [ns].

```ogn-client```

### Write raws stream to logfile

You can log the stream to a file. For example you can just pipe the output from above to a file.

```ogn-client > ogndata.log```

### Write the stream to TimescaleDB / PostgreSQL

[TimescaleDB](https://www.timescale.com/) is a very fast TSDB (time series database). It is based on the popular database [PostgreSQL](https://www.postgresql.org/). You can directly connect to this database. The connection string the ogn-client is using is ```postgres://postgres:postgres@localhost:5432/ogn```.

```ogn-client --target postgre-sql```

### Write the stream to QuestDB

[QuestDB](http://questdb.io) is another fast TSDB. By default QuestDB listen on port 9009 for new data.
The data format is the "InfluxDB Line Protocol", so you have to set the output format (which is by default "raw") to "influx" and pipe it to port 9009.

```ogn-client --format influx | nc localhost 9009```

### Write a raw logfile to QuestDB

If you created a raw logfile you can use it as source instead of the stream. Just set the source to "stdin".
Also set a timeout for nc (here: 1sec.) so the command finishes.

```cat ogndata.log | ogn-client --source stdin --format influx | nc -q 1 localhost 9009```


### Get help

If you need more informations about the command options just execute it with option "--help"

```ogn-client --help
Usage: ogn-client [OPTIONS]

Options:
  -s, --source <SOURCE>              specify input source [default: glidernet] [possible values: glidernet, stdin]
  -t, --target <TARGET>              specify output target [default: stdout] [possible values: stdout, postgre-sql, influx-db]
  -b, --batch-size <BATCH_SIZE>      maximum batch size for parallel stdin execution [default: 16384]
  -d, --database-url <DATABASE_URL>  database connection string [default: postgresql://postgres:postgres@localhost:5432/ogn]
  -i, --included <INCLUDED>          let pass only packets with given destination callsigns (comma separated)
  -e, --excluded <EXCLUDED>          drop packets with given destination callsigns (comma separated)
  -h, --help                         Print help
  -V, --version                      Print version
```

## Integrate OGN logger to OS (linux)

If you want to get the stream and save it on a daily basis (for example) for linux we have simple tools: systemd and logrotate.
First write a configuration for systemd and save it under "/etc/systemd/system/ogn.service". Modify the paths so the configuration
fits your system.

```
[Unit]
Description=Open Glider Network (OGN) data stream logger
After=network-online.target

[Service]
Type=simple
ExecStart=/home/pi/bin/ogn-client
StandardOutput=append:/var/log/ogn/stdout.log
StandardError=append:/var/log/ogn/stderr.log
Restart=always
RestartSec=1s

[Install]
WantedBy=multi-user.target
```

Now you have to enable and start the service

```
systemctl enable ogn
systemctl start ogn
```

To split and compress the logfile on a daily basis you can create a configuration file and safe it under "/etc/logrotate.d/ogn"

```
/var/log/ogn/stdout.log
/var/log/ogn/stderr.log {
  missingok
  daily
  dateext
  dateyesterday
  copytruncate
  compress
  rotate 40
}
```

