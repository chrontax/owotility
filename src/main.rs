use clap::{arg, command, value_parser, Command};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serial2::SerialPort;
use serialport::{SerialPortInfo, SerialPortType, UsbPortInfo};
use std::{
    array::from_fn,
    fmt::Display,
    fs::read_to_string,
    mem::{size_of, transmute},
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};
use usbd_human_interface_device::page::Keyboard;

const BAUD_RATE: u32 = 115200;

fn main() {
    let dev_arg = arg!(<DEVICE> "Device to use");
    let matches = command!()
        .subcommand(Command::new("devices").about("List available devices"))
        .subcommand(
            Command::new("configs")
                .about("Show key configs")
                .arg(dev_arg.clone()),
        )
        .subcommand(
            Command::new("binds")
                .about("Manage currently set keybinds")
                .arg(
                    arg!(-f --file <PATH> "Path to file with line-separated binds to set")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(dev_arg.clone()),
        )
        .subcommand(
            Command::new("save")
                .about("Save current configuration to flash")
                .arg(dev_arg.clone()),
        )
        .subcommand(
            Command::new("send")
                .about("Send something to the keypad")
                .arg(dev_arg.clone())
                .arg(arg!(<COMMAND> "Command to send")),
        )
        .subcommand(
            Command::new("status")
                .about("Show live status of the keypad")
                .arg(dev_arg.clone()),
        )
        .get_matches();
    if let Some((name, matches)) = matches.subcommand() {
        let dev = matches
            .get_one::<String>("DEVICE")
            .map(|dev| Device::new(dev));
        match name {
            "devices" => println!("{}", get_devices().join("\n")),
            "configs" => print_configs(dev.unwrap()),
            "binds" => {
                if let Some(path) = matches.get_one("file") {
                    set_binds(dev.unwrap(), path)
                } else {
                    print_binds(dev.unwrap())
                }
            }
            "save" => dev.unwrap().send("save"),
            "send" => dev
                .unwrap()
                .send(matches.get_one::<String>("COMMAND").unwrap()),
            "status" => print_status(dev.unwrap()),
            _ => unreachable!(),
        }
    }
}

fn get_devices() -> Vec<String> {
    serialport::available_ports()
        .unwrap()
        .iter()
        .filter_map(|p| {
            if let SerialPortInfo {
                port_name,
                port_type:
                    SerialPortType::UsbPort(UsbPortInfo {
                        vid: 0x5566,
                        pid: 0x0001,
                        ..
                    }),
            } = p
            {
                Some(port_name.clone())
            } else {
                None
            }
        })
        .collect()
}

fn print_configs(dev: Device) {
    for (i, c) in dev.get_configs().iter().enumerate() {
        println!("Key #{}: {}", i, c);
    }
}

fn print_binds(dev: Device) {
    let nodes = dev.get_nodes();

    fn get_binds(node: &Node, nodes: &[Node], prefix: String) -> Vec<String> {
        if let Some(key) = node.key {
            vec![format!("{}\t:\t{:?}", prefix, key)]
        } else {
            let mut vec = Vec::new();
            for (i, d) in ["L", "M", "R"].iter().enumerate() {
                if node.children[i] == 0 {
                    continue;
                }
                vec.append(&mut get_binds(
                    &nodes[node.children[i] as usize],
                    nodes,
                    prefix.clone() + d,
                ));
            }
            vec
        }
    }

    println!("{}", get_binds(&nodes[0], &nodes, String::new()).join("\n"));
}

fn set_binds(dev: Device, file: &PathBuf) {
    let binds = read_to_string(file).unwrap();
    dev.send("clear");
    for line in binds.lines() {
        dev.send(&format!("bind{}", line));
    }
    println!("Set");
}

fn print_status(dev: Device) {
    let mp = MultiProgress::new();
    let pbs: [_; 3] = from_fn(|_| {
        ProgressBar::new(dev.travel as u64).with_style(
            ProgressStyle::with_template("[{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("=# "),
        )
    });
    for pb in pbs.iter() {
        mp.add(pb.clone());
    }
    loop {
        let start = Instant::now();
        let depths = dev.get_depths();
        for i in 0..3 {
            pbs[i].set_position(depths[i] as u64);
        }
        sleep(Duration::from_millis(10).saturating_sub(start.elapsed()));
    }
}

struct Device {
    node_count: u16,
    travel: u16,
    serial: SerialPort,
}

impl Device {
    fn new(dev: &str) -> Self {
        let serial = SerialPort::open(dev, BAUD_RATE).unwrap();
        serial.write_all(b"consts").unwrap();
        let mut buf = [0; 6];
        serial.read_exact(&mut buf).unwrap();
        let [node_count, travel, _] = unsafe { transmute(buf) };
        Self {
            node_count,
            travel,
            serial,
        }
    }

    fn get_configs(&self) -> [KeyConfig; 3] {
        self.serial.write_all(b"config").unwrap();
        let mut buf = [0; size_of::<KeyConfig>() * 3];
        self.serial.read_exact(&mut buf).unwrap();
        unsafe { transmute(buf) }
    }

    fn get_nodes(&self) -> Vec<Node> {
        self.serial.write_all(b"nodes").unwrap();
        let mut buf = Vec::with_capacity(self.node_count as usize * size_of::<Node>());
        unsafe { buf.set_len(self.node_count as usize * size_of::<Node>()) }
        self.serial.read_exact(&mut buf).unwrap();
        unsafe { transmute(buf) }
    }

    fn get_depths(&self) -> [u16; 3] {
        self.serial.write_all(b"depth").unwrap();
        let mut buf = [0; 6];
        self.serial.read_exact(&mut buf).unwrap();
        unsafe { transmute(buf) }
    }

    fn send(&self, cmd: &str) {
        self.serial.write_all(cmd.as_bytes()).unwrap();
    }
}

struct KeyConfig {
    rt_up: u16,
    rt_down: u16,
    min: u16,
    max: u16,
}

impl Display for KeyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rt_up: {}, rt_down: {}, min: {}, max: {}",
            self.rt_up, self.rt_down, self.min, self.max
        )
    }
}

struct Node {
    children: [u16; 3],
    key: Option<Keyboard>,
}
