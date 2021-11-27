# snes-tello

![A black and white quadcopter labeled Tello with propeller guards](./assets/irl.png)

control your [Ryze Tello](https://www.ryzerobotics.com/tello) drone with a Super Nintendo controller!

a quick weekend project :P

## usage

- get an esp32
- get platformio
- `pio run -t upload`
- connect the controller pins to your esp32

| esp32 | SNES  |
|:-----:|:-----:|
| vcc   | vcc   |
| D4    | clock |
| D15   | latch |
| D21   | data  |
| gnd   | gnd   |

- go to the driver
- `cargo run -- --help`
- read it and also make sure you have ffmpeg
- set the options and run

## thanks

- [Alexander89/rust-tello](https://github.com/Alexander89/rust-tello)
- $FAMILY_MEMBER who thought this was a good birthday gift

## license

MIT, see `LICENSE` file

