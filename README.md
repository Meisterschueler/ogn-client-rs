# ogn-client-rs

A client for the [Open Glider Network](http://wiki.glidernet.org/) written in Rust.

## Example Usage

### Get the raw stream

Just start the client. It will connect to the OGN data stream and print it out. Every message received is prefixed by a timestamp [ns].

```ogn-client```

### Write raws stream to logfile

You can log the stream to a file. For example you can just pipe the output from above to a file.

```ogn-client > ogndata.log```

### Write the stream to QuestDB

[QuestDB](http://questdb.io) is an extreme fast TSDB (time series database). By default QuestDB listen on port 9009 for new data.
The data format is the "InfluxDB Line Protocol", so you have to set the output format (which is by default "raw") to "influx" and pipe it to port 9009.

```ogn-client --format influx | nc localhost 9009```

### Write a raw logfile to QuestDB

If you created a raw logfile you can use it as source instead of the stream. Just set the source to "console".
Also set a timeout for nc (here: 1sec.) so the command finishes.

```cat ogndata.log | ogn-client --source console --format influx | nc -q 1 localhost 9009```

### Filter the stream

You can use the options "includes" or "excludes". If you want to get beacons from receiver "Koenigsdf" or "Koenigsd2" the command looks like

```ogn-client --includes Koenigsdf,Koenigsd2```

### Get help

If you need more informations about the command options just execute it with option "--help"

```ogn-client --help
Usage: ogn-client [OPTIONS]

Options:
  -s, --source <SOURCE>      specify input source [default: glidernet] [possible values: glidernet, console]
  -f, --format <FORMAT>      specify output format [default: raw] [possible values: raw, json, influx]
  -i, --includes <INCLUDES>  proceed only APRS messages including a substring - format: comma separated strings
  -e, --excludes <EXCLUDES>  don't proceed APRS messages including a substring - format: comma separated strings
  -h, --help                 Print help information
  -V, --version              Print version information
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

