# zquery

`zquery` is a tool for running SQL queries on the output of various Unix tools. It lets you query the output of various Unix tools in a way that's easy to work with. It's powered by [Apache DataFusion](https://datafusion.apache.org/) and [jc](https://github.com/kellyjonbrazil/jc) which means it works way better than it has any right to.

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

//We can go remote too.
>>select * from who(host('some_ssh_host'));
+------+-------+-------+------------------+------------+
| user | event | tty   | time             | epoch      |
+------+-------+-------+------------------+------------+
| root |       | pts/0 | 2024-09-22 19:51 | 1727049060 |
+------+-------+-------+------------------+------------+


// We can join across commands too and do all the normal SQL stuff.
>> SELECT
    ps.user,
    ps.pid,
    LEFT(ps.command, 50) AS command,
    ps.cpu_percent,
    ps.mem_percent,
    up.uptime_days,
    up.load_1m,
    up.load_5m,
    up.load_15m
FROM
    ps() as ps
CROSS JOIN
    uptime() AS up
WHERE
    ps.cpu_percent > 1.0
ORDER BY
    ps.cpu_percent DESC
LIMIT 5;
+---------------+-------+----------------------------------------------------+-------------+-------------+-------------+---------+---------+----------+
| user          | pid   | command                                            | cpu_percent | mem_percent | uptime_days | load_1m | load_5m | load_15m |
+---------------+-------+----------------------------------------------------+-------------+-------------+-------------+---------+---------+----------+
| zackmaril     | 18477 | /Applications/Firefox.app/Contents/MacOS/firefox - | 20.3        | 3.0         | 29          | 3.21    | 3.54    | 3.79     |
| _windowserver | 374   | /System/Library/PrivateFrameworks/SkyLight.framewo | 14.5        | 0.3         | 29          | 3.21    | 3.54    | 3.79     |
| zackmaril     | 18979 | /Applications/Firefox.app/Contents/MacOS/plugin-co | 13.0        | 1.6         | 29          | 3.21    | 3.54    | 3.79     |
| zackmaril     | 48158 | /Applications/Firefox.app/Contents/MacOS/plugin-co | 11.5        | 1.4         | 29          | 3.21    | 3.54    | 3.79     |
| zackmaril     | 18692 | /Applications/Firefox.app/Contents/MacOS/plugin-co | 7.0         | 0.5         | 29          | 3.21    | 3.54    | 3.79     |
+---------------+-------+----------------------------------------------------+-------------+-------------+-------------+---------+---------+----------+
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
- `ls('.')` - Returns a table of files in the given directory.
- `stat('.')` - Returns a table of file information for the given path.
- `df('.')` - Returns a table of disk space information.
- `du('.')` - Returns a table of disk usage information.
- `env()` - Returns a table of environment variables.
- `date()` - Returns a table of date and time information.
- `dir('.')` - Returns a table of directory information. (Windows only)
- `dpkg_list()` - Returns a table of dpkg packages. 
- `file('.')` - Returns a table of file information. 
- `find('.')` - Returns a table of files and directories.  (TODO: broken)
- `free()` - Returns a table of free memory information.

To query remote servers, you can use the `host()` function as the first argument to any of the above commands.

For example: 
```
>> select * from ps(host('some_ssh_host'));
>> select * from ls(host('some_ssh_host'), '/home/some_user/');
```
## SQL Support

Currently, `zquery` supports the `SELECT` statement in general. Nested, windows, joins, aggregates are all supported. Datafusion is great! Support for `CREATE TABLE` with `INSERT`, `UPDATE`, `DELETE` and `DROP` statements is planned via a local sqlite3 database. Additionally, support for streaming sql is in the works. 

## Limitations 

* Currently it relies on short names in your `ssh` config for simplicity of using `host('server_name')`. If you want to use `zquery` with your own ssh config you can do so but you'll have to make sure your config file has all the info ssh2 needs to get into the server.
* Additionally, `zquery` experts there to be an `id_rsa` file in your `~/.ssh` directory. It shouldn't be too hard to add support for other key types or other methods of authentication but that isn't done yet.
* The ssh commands are implemented as user defined table functions in datafusion. This is convenient in some ways, but a limitation right now is that it's not possible to pass columns as arguments to these functions. So any arguments to the functions need to be literals. This is a work in progress, and may be possible in the future once I learn more about how datafusion works.

## Contributing/License

`zquery` is licensed under the MIT License. `zquery` is a work in progress so contributions are welcome with the understanding that things might be unstable and breaking and that this is a big experiment with a vision I am working towards. So if a PR languishes or is not accepted at the moment, that's because I'm one person focusing on trying to get the vision down. 

## Roadmap 

- [ ] Expand command list. 
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
