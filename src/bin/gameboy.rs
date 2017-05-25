#[macro_use]
extern crate clap;
extern crate gameboy;
extern crate minifb;

use std::path::Path;
use clap::{Arg, App};
use minifb::{Key, Scale, WindowOptions, Window};
use gameboy::vm::VM;
use gameboy::bootrom::Bootrom;
use gameboy::cartridge::Cartridge;
use gameboy::interconnect::Interconnect;
use gameboy::device::{self, Device};
use gameboy::symbols::Symbols;

struct ConsoleDevice {
    buffer: Box<[u32]>,
    window: Window,

    width: usize,
    height: usize,

    buffer_set: bool,
}

impl ConsoleDevice {
    fn new(window: Window, width: usize, height: usize) -> Self {
        ConsoleDevice {
            buffer: vec![0; width * height].into_boxed_slice(),
            window: window,
            width: width,
            height: height,
            buffer_set: false,
        }

    }
}

impl Device for ConsoleDevice {
    fn update(&mut self) {
        if self.buffer_set {
            self.window.update_with_buffer(&*self.buffer);
            self.buffer_set = false;
        }
    }

    fn set_frame_buffer(&mut self, buffer: &[u32]) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = (y * self.width) + x;
                self.buffer[index as usize] = buffer[index as usize];
            }
        }
        self.buffer_set = true;
    }

    fn key_down(&self, key: device::Key) -> bool {
        let key = match key {
            device::Key::Up => Key::Up,
            device::Key::Down => Key::Down,
            device::Key::Left => Key::Left,
            device::Key::Right => Key::Right,
            device::Key::Backspace => Key::Backspace,
            device::Key::Enter => Key::Enter,
            device::Key::Z => Key::Z,
            device::Key::X => Key::X,
        };

        self.window.is_key_down(key)
    }

    fn running(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}

fn main() {
    let matches = App::new("Gameboy Emulator")
        .version(crate_version!())
        .author("tompko  <tompko@gmail.com>")
        .about("Emulates the Game Boy language")
        .arg(Arg::with_name("INPUT")
                 .help("Sets the cartridge file to use")
                 .required(true)
                 .index(1))
        .arg(Arg::with_name("boot-rom")
                 .help("Sets the boot rom to use")
                 .short("b")
                 .long("boot-rom")
                 .required(true)
                 .takes_value(true))
        .arg(Arg::with_name("sym-file")
                 .help("Sets the symbol file to use")
                 .short("s")
                 .long("sym")
                 .takes_value(true))
        .arg(Arg::with_name("debug")
                 .help("If present, starts in debugging mode")
                 .short("d")
                 .long("debug")
                 .takes_value(false))
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let boot_rom_file = matches.value_of("boot-rom").unwrap();
    let cartridge = Cartridge::load(Path::new(input_file)).unwrap();
    let start_in_debug = matches.is_present("debug");

    let boot_rom = Bootrom::load(Path::new(boot_rom_file)).unwrap();

    let symbols = if let Some(sym_file) = matches.value_of("sym-file") {
        Symbols::load(sym_file).unwrap()
    } else {
        Symbols::default()
    };

    let interconnect = Interconnect::new(boot_rom, cartridge);
    let width = interconnect.get_width();
    let height = interconnect.get_height();

    let mut vm = VM::new(interconnect, start_in_debug, symbols);

    let window_options = WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: Scale::X2,
    };

    let window = Window::new("GBrs", width, height, window_options).unwrap();

    let mut device = ConsoleDevice::new(window, width, height);

    vm.run(&mut device);
}
