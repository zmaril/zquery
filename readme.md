# zquery

`zquery` is a tool for running SQL queries on the output of various Unix tools. I got tired of writing joins by hand in bash so I made this. It lets you query the output of various Unix tools in a way that's easy to work with, for whichever server you might have access to via a SSH config shortname. It's powered by [Apache DataFusion](https://datafusion.apache.org/) and [jc](https://github.com/kellyjonbrazil/jc) which means it works way better than you might expect.

```
$ zquery 
>> select * from ps() order by pid limit 1;
+---------------+-------------+-------------+-----+-------+---------+------+----------+-----+
| command       | cpu_percent | mem_percent | pid | rss   | started | stat | time     | tty |
+---------------+-------------+-------------+-----+-------+---------+------+----------+-----+
| /sbin/launchd | 0.0         | 0.0         | 1   | 15664 | 24Aug24 | Ss   | 34:45.79 |     |
+---------------+-------------+-------------+-----+-------+---------+------+----------+-----+

>> SELECT left(command,70) as cmd, cpu_percent FROM ps() WHERE cpu_percent > 0.5 order by cpu_percent desc, cmd;
+------------------------------------------------------------------------+-------------+
| cmd                                                                    | cpu_percent |
+------------------------------------------------------------------------+-------------+
| /Applications/Firefox.app/Contents/MacOS/plugin-container.app/Contents | 99.0        |
| /Applications/iTerm.app/Contents/MacOS/iTerm2                          | 17.0        |
| /System/Library/CoreServices/ReportCrash agent                         | 9.9         |
| /System/Library/PrivateFrameworks/SkyLight.framework/Resources/WindowS | 7.9         |
| /System/Library/PrivateFrameworks/CoreSymbolication.framework/coresymb | 6.6         |
| /Applications/Firefox.app/Contents/MacOS/plugin-container.app/Contents | 3.2         |
| /Applications/Firefox.app/Contents/MacOS/firefox -foreground           | 2.0         |
| /Applications/Cursor.app/Contents/Frameworks/Cursor Helper (Plugin).ap | 1.8         |
| /Applications/Firefox.app/Contents/MacOS/plugin-container.app/Contents | 1.3         |
| /Applications/Firefox.app/Contents/MacOS/plugin-container.app/Contents | 0.9         |
| /System/Library/CoreServices/Siri.app/Contents/MacOS/Siri launchd      | 0.9         |
| /Library/Application Support/iStat Menus 5/iStat Menus Status.app/Cont | 0.8         |
| /usr/sbin/cfprefsd daemon                                              | 0.6         |
+------------------------------------------------------------------------+-------------+

>> select * from uptime();
+-----------------+-------+---------+---------+----------+-----------+-------------+-------------+-------------+--------------+----------------+----------------------+
| uptime          | users | load_1m | load_5m | load_15m | time_hour | time_minute | time_second | uptime_days | uptime_hours | uptime_minutes | uptime_total_seconds |
+-----------------+-------+---------+---------+----------+-----------+-------------+-------------+-------------+--------------+----------------+----------------------+
| 25 days, 19 hrs | 2     | 1.85    | 2.32    | 2.4      | 4         | 56          |             | 25          | 0            | 0              | 2160000              |
+-----------------+-------+---------+---------+----------+-----------+-------------+-------------+-------------+--------------+----------------+----------------------+


//n.b. We aren't parsing time/date yet.
>> select * from who() order by time
+-----------+--------+---------+--------------+-------+
| user      | event  | tty     | time         | epoch |
+-----------+--------+---------+--------------+-------+
|           | reboot |         | Aug 24       |       |
| zackmaril |        | console | Aug 24 09:57 |       |
| zackmaril |        | ttys009 | Aug 25 09:57 |       |
| zackmaril |        | ttys004 | Aug 26 08:39 |       |
| zackmaril |        | ttys010 | Aug 27 09:52 |       |
| zackmaril |        | ttys003 | Aug 27 18:33 |       |
| zackmaril |        | ttys021 | Aug 30 07:46 |       |
| zackmaril |        | ttys022 | Aug 30 07:56 |       |
| zackmaril |        | ttys002 | Sep 1 03:59  |       |
| zackmaril |        | ttys007 | Sep 12 23:10 |       |
| zackmaril |        | ttys000 | Sep 19 03:05 |       |
| zackmaril |        | ttys001 | Sep 19 04:59 |       |
| zackmaril |        | ttys018 | Sep 4 07:59  |       |
| zackmaril |        | ttys025 | Sep 6 17:08  |       |
| zackmaril |        | ttys020 | Sep 8 18:58  |       |
+-----------+--------+---------+--------------+-------+
```



## Installation

To install `zquery`, ensure you have Rust's package manager, Cargo, installed. You'll also need `jc` (JSON CLI output utility) installed locally.

1. Install `zquery` using Cargo:

   ```bash
   cargo install zquery
   ```

2. Install `jc` (if not already installed):

   ```bash
   pip install jc
   ```

## Supported Commands

`zquery` currently supports the following commands:

- `ps()` - Returns a table of processes.
- `uptime()` - Returns a row of uptime information.
- `who()` - Returns a table of who is and was on the system.

With more to come soon! 

## Contributing/License

`zquery` is licensed under the MIT License. `zquery` is a work in progress so contributions are welcome with the understanding that things might be unstable and breaking and that this is a big experiment and that I have a vision I am working towards.

## Roadmap 

- [ ] Expand command list. 
- [ ] Add remote command usage. 
- [ ] Add pgwire.
- [ ] Add a TUI. 
- [ ] Make it easy to install.
- [ ] Add tests. 
- [ ] Add CI.
- [ ] Branch protection and pull requests and versioning.
- [ ] Make a website for it with better documentation and with real time examples.
- [ ] Set up a discord.
- [ ] Figure out how to get streaming working with datafusion. 
- [ ] Get bpftrace working ala [`bpfquery`](https://bpfquery.com).
- [ ] Figure out a better way than `jq` to handle the output of `jc` for converting from JSON to NDJSON.Or, hey, don't convert from JSON to NDJSON at all! Just have arrow parse the JSON directly somehow.
