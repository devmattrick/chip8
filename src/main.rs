use std::io;
use std::io::prelude::*;
use std::fs::File;

mod chip8;

fn main() {
    let mut rom: [u8; 3584] = [0; 3584];
    let mut file = File::open("test.rom").unwrap();

    file.read(&mut rom).unwrap();

    let mut vm = chip8::VM::new();
    vm.load(rom);
    vm.cycle();
}
