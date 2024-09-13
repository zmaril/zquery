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

$ zquery -e "SELECT p.command, p.cpu_usage, u.info FROM ps() as p JOIN whoami() as u ON p.user = u.user"
+-----------------------------------------------------------------------------------+
| command  | cpu_usage | info |
|-----------------------------------------------------------------------------------|
| /usr/lib/systemd/systemd --user | 0.0 | root |
| /usr/lib/systemd/systemd --system | 0.0 | root |
| /usr/lib/systemd/systemd-networkd | 0.0 | root |
| /usr/lib/systemd/systemd-resolved | 0.0 | root |
| /usr/lib/systemd/systemd-timesyncd | 0.0 | root |
| /usr/lib/systemd/systemd-logind | 0.0 | root |
+-----------------------------------------------------------------------------------+
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
