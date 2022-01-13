# turtlebot2

To drive Turtlebot2 (a.k.a Kobuki).

<br/>
<br/>

## References

- https://yujinrobot.github.io/kobuki/enAppendixProtocolSpecification.html
- https://crates.io/crates/serialport

<br/>

## Prerequisites

Since the serialport crate requires some packages:
```sh
$ sudo apt install libudev-dev pkg-config
```

Also the user should be a part of the dialout group in Linux:
```sh
$ sudo adduser ${USER} dialout

# or 

$ sudo usermod -aG dialout ${USER}
```

<br/>

## Demo

A demo shows the basic usage of this crate (only tested in Linux!):
```sh
$ cd examples/basic
$ cargo run
```
