# turtlebot2

To drive Turtlebot2 (a.k.a Kobuki).

<br/>
<br/>

## References

- https://yujinrobot.github.io/kobuki/enAppendixProtocolSpecification.html
- https://crates.io/crates/serialport

<br/>

## Prerequisites

Since the seriaport crate requires some packages:
```sh
$ sudo apt install libudev-dev pkg-config
```

Also the user should be in the dialout group of Linux:
```sh
$ sudo adduser ${USER} dialout

# or 

$ sudo usermod -aG dialout ${USER}
```

<br/>

