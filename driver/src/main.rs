extern crate serial;

use std::{io::{self, Write}, process::{Command, Stdio}, sync::mpsc::{self, TrySendError}, thread::{self, JoinHandle}, time::{Duration, Instant}};

use clap::{App, Arg};
use serial::prelude::*;
use tello::{Drone, Message};

fn main() {
    let matches = App::new("tello-driver")
        .arg(
            Arg::with_name("serial")
                .long("serial")
                .takes_value(true)
                .help("serial port to use, e.g. /dev/ttyUSB1"),
        )
        .arg(
            Arg::with_name("drone_ip")
                .long("drone-ip")
                .short("i")
                .takes_value(true)
                .help("the IP address of the tello drone"),
        )
        .get_matches();

    let drone_ip = matches.value_of("drone_ip").expect("missing argument");
    let port_name = matches.value_of("serial").expect("missing argument");
    let (_, controller_rx) = make_controller_thread(port_name);
    drone_loop(&format!("{}:8889", drone_ip), controller_rx);
}

fn drone_loop(addr: &str, controller_rx: mpsc::Receiver<ControllerState>) {
    let mut drone = Drone::new(addr);
    println!("Connecting to {}...", addr);
    drone.connect(11111);
    drone.start_video().unwrap();
    drone.send_date_time().unwrap();
    let mut ffplay_channel = {
        let mut proc = Command::new("ffplay").arg("-").stdin(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        proc.stdin.take().unwrap()
    };
    loop {
        let loop_time = Instant::now();
        {
            // this is done for drone_meta to get populated
            // to not timeout, and so we can poke at the video data
            let msg = drone.poll();
            if let Some(Message::Frame(id, buf)) = msg {
                println!("recv frame id {}", id);
                ffplay_channel.write(&buf).unwrap();
            }
            if let Some(flight_data) = drone.drone_meta.get_flight_data() {
                println!("Battery: {}%", flight_data.battery_percentage);
                if let Ok(state) = controller_rx.recv_timeout(Duration::from_millis(10)) {
                    dbg!(&state);
                    if state.start && state.select {
                        if flight_data.fly_time > 0 {
                            drone.take_off().unwrap();
                        } else {
                            drone.stop_land().unwrap();
                        }
                    } else if state.start {
                        drone.rc_state.start_engines();
                    }

                    drone.rc_state.stop_turn();
                    if state.right {
                        drone.rc_state.go_cw();
                    } else if state.left {
                        drone.rc_state.go_ccw();
                    }

                    drone.rc_state.stop_up_down();
                    if state.x {
                        drone.rc_state.go_up();
                    } else if state.b {
                        drone.rc_state.go_down();
                    }

                    drone.rc_state.stop_forward_back();
                    if state.pad_up {
                        drone.rc_state.go_forward();
                    } else if state.pad_down {
                        drone.rc_state.go_back();
                    }

                    drone.rc_state.stop_left_right();
                    if state.pad_left {
                        drone.rc_state.go_left();
                    } else if state.pad_right {
                        drone.rc_state.go_right();
                    }
                }
            } else {
                eprintln!("no flight data!");
            }
        }
        thread::sleep(
            Duration::from_millis(33)
                .checked_sub(loop_time.elapsed())
                .unwrap_or_else(|| -> Duration { Duration::from_millis(1) }),
        );
    }
}
fn make_controller_thread(port_name: &str) -> (JoinHandle<()>, mpsc::Receiver<ControllerState>) {
    let (tx, rx) = mpsc::sync_channel(2);

    let port_name = port_name.to_string();
    let thread = thread::spawn(move || {
        let mut port = init_serial(&port_name);
        let mut poll_count = 0;
        loop {
            // the client sends magic "CKIE" then 12 bytes with the button states
            // 12 + 4 = 16

            let mut magic_stack = vec![];
            loop {
                let byte = read_byte(&mut port).unwrap();
                magic_stack.push(byte);
                if magic_stack.ends_with("CKIE".as_bytes()) {
                    if let Some(btns) = magic_stack.get(0..12) {
                        let pressed: Vec<_> = btns.iter().map(|&x| x != 255).collect();
                        let state = ControllerState {
                            b: pressed[0],
                            y: pressed[1],
                            select: pressed[2],
                            start: pressed[3],
                            pad_up: pressed[4],
                            pad_down: pressed[5],
                            pad_left: pressed[6],
                            pad_right: pressed[7],
                            a: pressed[8],
                            x: pressed[9],
                            left: pressed[10],
                            right: pressed[11],
                        };
                        // discard the first 20 samples since weird things happen
                        if poll_count > 20 {
                            let result = tx.try_send(state);
                            // if it succeeded or the buffer was full we don't really care
                            match result {
                                Err(TrySendError::Disconnected(_)) => {
                                    result.unwrap();
                                },
                                _ => {}
                            }
                        }
                        poll_count += 1;
                        break;
                    }
                }
            }
        }
    });
    (thread, rx)
}

fn init_serial(port_name: &str) -> impl SerialPort {
    let mut port = serial::open(port_name).unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .unwrap();
    port
}

fn read_byte(port: &mut dyn SerialPort) -> io::Result<u8> {
    let mut buf = vec![0; 1];
    port.read_exact(&mut buf)?;

    Ok(*buf.get(0).unwrap())
}

#[derive(Debug)]
struct ControllerState {
    b: bool,
    y: bool,
    select: bool,
    start: bool,
    pad_up: bool,
    pad_down: bool,
    pad_left: bool,
    pad_right: bool,
    a: bool,
    x: bool,
    left: bool,
    right: bool,
}
