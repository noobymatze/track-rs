# Track

Track is a very simple command line tool to track your time with a 
backing [Redmine][redmine] server.


## Installation

You can install track by downloading the binary, making it executable
and putting it some place that is on your path.

```sh
$ wget https://github.com/noobymatze/track-rs/releases/download/v0.2.0/track
$ chmod +x track
$ sudo mv track /usr/bin
```


## Example

Before we do anything, your Redmine instance needs to be configured to
allow working with it via REST [1][rest]. Then, you need to generated
an API key for your user. We are ready to use track then.

```
$ track login -u myuser -b https://myredmine.server
Password: 
$ track list
+---------+-------+-------------+---------+
| Project | Issue | Hours (∑ 0) | Comment |
+---------+-------+-------------+---------+
$ track
Issue (leave empty for project only): 1
Comment: 14:00 - 15:00
Activity: Something
$ track list
+-----------------+-------+-------------+----------------+
| Project         | Issue | Hours (∑ 1) | Comment        |
+-----------------+-------+-------------+----------------+
| Track dev stuff | #1    |           1 | 14:00 - 15:00  |
+-----------------+-------+-------------+----------------+
```

If you decide to write start and finish time as the first part of the
comment, the duration will be calculated automatically. Otherwise you
will need to provide it afterwards. Beware though, it is optimized for
quarter-hourly increments.


## Notes

This is a re-implementation of [track][track] in Rust. You should
start using the binary from this project, since it provides some nice
UX improvements.


## Roadmap

- [ ] Show a weekly visualisation of your time
- [ ] Write tests
- [x] Allow to provide a flag --yesterday to track time from the day before.


[redmine]: https://www.redmine.org/
[rest]: https://www.redmine.org/projects/redmine/wiki/Rest_api
[track]: https://github.com/noobymatze/track
