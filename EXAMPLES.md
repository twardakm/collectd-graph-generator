# Examples for collectd-graph-generator

## 0.1.0

### RSS of multiple processes

If you have enabled **collectd** plugin **processes** with multiple entries it is possible to see memory usage of all processes in one chart.

First example was generated while collectd was configured to track 5 processes, but only 4 were running:

```bash
$ cargo run -- \
--input /var/lib/collectd/marcin-manjaro/ \
--out "1 - RSS of 4 processes.png" \
-w 1024 \
-h 768 \
--start 1604224000 \
--end 1604225000
```

And second one is showing a chart with starting new process:

```bash
$ cargo run -- \
--input /var/lib/collectd/marcin-manjaro/ \
--out "2 - RSS of 5 processes.png" \
-w 1024 \
-h 768 \
--start 1604225000 \
--end 1604226000
```

## 0.1.1

### RSS of multiple processes via SSH

In version `0.1.1` support of gathering data over SSH was added:

```bash
$ cargo run -- \
--input marcin@192.168.0.163:/var/lib/collectd/marcin-manjaro/ \
--out "1 - RSS of multiple processes via SSH.png" \
-w 1024 \
-h 768 \
--start 1604253000 \
--end 1604254000
```

## 0.1.2

### Human readable timespans

In version `0.1.2` support for human readable timespan was added:

```bash
$ cargo run -- \
--input marcin@192.168.0.163:/var/lib/collectd/marcin-manjaro/ \
--out "1 - RSS of multiple processes via SSH.png" \
-w 1024 \
-h 768 \
-t "last 2 hours"
```

Currently supported are:
- seconds
- minutes
- hours
- days
- weeks
- months
- years