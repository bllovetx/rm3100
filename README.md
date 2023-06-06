# RM3100

## Introduction

This crate encapsulate rm3100 (magnetic sensor, communicate through spi) based on hal

## Components

### packet

Spi data packet, contain one byte r/w address and data(all zero when read).

mainly used forhandle data type conversion

### mincircularbuffer

minmum circular buffer, contains an array and two "pointer", only has `pop`, `push` and `clear`

## Usage

To flash app(as an example):

run openocd and then run

```terminal
cargo run --example app
```

in this repository

## Examples

### app

This app realize an embedded rm3100(magnetic sensor) server, which based on rtic, communicated with rm3100 through spi, communicated with PC through usb and can be triggerred by input ttl. It runs on stm32f3discovery, but can be easily transformed to other boards.

PC end rpc server communicate with this app see [RM3100_RPC](https://github.com/bllovetx/RM3100_RPC)

USB expose two Interface, one CDC Interrupt and one CDC DATA. To W/R, use Endpoint 0x2/0x82

#### protocal:

| write | function & return |
| - | - |
| 0x80  | mag(five bytes): first byte 0 if no data available last four bytes i32 mag |
| 0x81  | is oveflow?(one byte): 0 if not overflow |
|0x82   |clear overflow(one byte): 1 after finish |
|0x83   | clear buffer(one byte): 1 after finish |

#### Performance

on different board tested respond(trigger output) 5-10us