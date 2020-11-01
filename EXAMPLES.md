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