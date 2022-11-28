`az-joystickshield-on-avr`
==================

This is a serial interface server for reading input from the [AZ Delivery PS2 Joystick Shield Game Pad Keypad V2.0](https://www.az-delivery.de/products/azdelivery-joystick-ky-023-keypad-gamepad-shield-ps2-fur-arduino-uno-r3-mega2560-leonardo-duemilanove) via an AVR based MCU (so far tested on Arduino Nano, but other Arduino platforms should be fairly easy).

![Image of Arduino Nano and Shield connected by wires](./doc/arduino-nano-on-shield.jpg)

## Features

* mini-server running on Arduino, serving the host via RX/TX (RS-232 USART built into Arduino)
* accepts commands from host for configuration and requesting data
* text format for debugging (default) and binary format for fast and compact data transmission
* multple transmission modes:
  - on-event (default): Sends data when something changed
  - on-request: Sends data only when requested by host
  - continuous: Sends data always when it becomes available


## Target plaforms

So far this project is tested on **Arduino Nano**. It was started with [avr-hal-template](https://github.com/Rahix/avr-hal-template), and *should* support the following platforms (target configurations exist in the [avr-specs](avr-specs) directory, the currently active one is referenced in [.cargo/Cargo.toml](.cargo/config.toml)):
 - Arduino Leonardo
 - Arduino Mega 2560
 - Arduino Mega 1280
 - Arduino Nano
 - Arduino Nano New Bootloader (Manufactured after January 2018)
 - Arduino Uno
 - SparkFun ProMicro
 - Adafruit Trinket
 - Adafruit Trinket Pro

## Usage
Install [`ravedude`]:

```bash
cargo install ravedude
```

Simply build and run, with your Arduino connected via USB (see below):

```bash
cargo run
```

When started, the server will print a welcome message:



## Connecting the shield to the Arduino

The software uses a pinout that should be suitable for most Arduinos. In principle, it uses A0,A1 for the analog stick's X and Y axes, and D2..D5 for the button inputs (configured with internal pull up input).

### Arduino Uno ###
**Not yet tested**

In theory, this should be as easy as plugging the shield directly onto the Arduino Uno, as per the joystick shield's manual.

### Arduino Nano ###

Since the shield was constructed for the Arduino Uno, connecting it to the Nano requires to properly wire it, as the Nano's pinout is not compatible with shields.

Connection is easy though.

You'll also find that there are multiple ways to connect the shild; what's shown here is one of them. Because there's no complete pinout description for the shield (at least I couldn't find one quickly), we'll state the pinout here using this image:

![Pinout for shield](doc/shield-pinout.jpg)

I simply named the relevant connectors S, T and U. S and T offer 6 pin slots, U has 8, from left to right.

| what     | Arduino pin | shield pin |
|----------|-------------|------------|
| VCC      | 5V          |     S3     |
| GND      | GND         |     S5     |
| x axis   | A0          |     T2     |
| y axis   | A1          |     T1     |
| button A | D2          |     U6     |
| button B | D3          |     U5     |
| button C | D4          |     U4     |
| button D | D5          |     U3     |

Note that the board features additional buttons F and E, which I didn't connect, as I didn't need them for now (feel free to suggest)

## License
Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
