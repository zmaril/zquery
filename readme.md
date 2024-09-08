# zquery

`zquery` is a tool for running SQL queries on the output of various Unix tools. I got tired of writing joins by hand in bash so I made this. It lets you query the output of various Unix tools in a way that's easy to work with, for whichever server you might have access to via a SSH config shortname. It's powered by [Apache DataFusion](https://datafusion.apache.org/) and [jc](https://github.com/kellyjonbrazil/jc) which means it works way better than you might expect.

```
$ zquery -e "SELECT * FROM ps()"
+-----------------------------------------------------------------------------------+
| command  | user | pid | cpu_usage | mem_usage | vsz  | rss  | tty  | stat | start | time |
|-----------------------------------------------------------------------------------|
| /usr/lib/systemd/systemd --user | root | 1 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd --system | root | 2 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-networkd | root | 3 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-resolved | root | 4 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-timesyncd | root | 5 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-logind | root | 6 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 7 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 8 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 9 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 10 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 11 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 12 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 13 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 14 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 15 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
| /usr/lib/systemd/systemd-user-sessions | root | 16 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
+-----------------------------------------------------------------------------------+

$ zquery -e "SELECT command FROM ps() WHERE cpu_usage > 0.5"
+-----------------------------------------------------------------------------------+
| command  |
|-----------------------------------------------------------------------------------|
| /usr/lib/systemd/systemd --user |
| /usr/lib/systemd/systemd --system |
| /usr/lib/systemd/systemd-networkd |
| /usr/lib/systemd/systemd-resolved |
| /usr/lib/systemd/systemd-timesyncd |
| /usr/lib/systemd/systemd-logind |
| /usr/lib/systemd/systemd-user-sessions |
+-----------------------------------------------------------------------------------+

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

$ zquery -e "SELECT * FROM ps('my_remote_server') WHERE user = 'root'" 
+-----------------------------------------------------------------------------------+
| command  | user | pid | cpu_usage | mem_usage | vsz  | rss  | tty  | stat | start | time |
|-----------------------------------------------------------------------------------|
| /usr/bin/sudo -i | root | 1 | 0.0 | 0.0 | 225288 | 13120 | ? | Ss | 2024-02-20 12:34:56 | 00:00:01 |
+-----------------------------------------------------------------------------------+

//join between two remote servers
$ zquery -e "SELECT p.command, p.cpu_usage, u.info FROM ps('my_remote_server') as p JOIN whoami('my_other_remote_server') as u ON p.user = u.user"
+-----------------------------------------------------------------------------------+
| command  | cpu_usage | info |
|-----------------------------------------------------------------------------------|
| /usr/bin/sudo -i | 0.0 | root |
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

## Contributing/License

`zquery` is licensed under the MIT License. `zquery` is a work in progress so contributions are welcome with the understanding that things might be unstable and breaking and that this is a big experiment and that I have a vision I am working towards.

## Roadmap 

- [ ] Make all of the above true. 
- [ ] Make repl and file eval work.
- [ ] Add in more commands and outputs. 
- [ ] Add a TUI. 
- [ ] Add pgwire.
- [ ] Make it easy to install.
- [ ] Add tests. 
- [ ] Add CI.
- [ ] Branch protection and pull requests and versioning.
- [ ] Make a website for it with better documentation and with real time examples.
- [ ] Set up a discord.
- [ ] Figure out how to get streaming working with datafusion. 
- [ ] Get bpftrace working ala [`bpfquery`](https://bpfquery.com).
