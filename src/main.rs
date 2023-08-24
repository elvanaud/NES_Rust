#![allow(non_snake_case)]

mod CPU;
mod Bus_NES;

use crate::CPU::*;
use crate::Bus_NES::*;

use std::fs::File;
use std::io::Read;
pub struct Cartridge{
}
impl Cartridge{
    fn new(path: &str) -> Cartridge{
        let mut file = File::open(path).expect("Can't find ROM file !");
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).expect("Failed to read ROM file");
        //println!("{contents:?}");
        Cartridge {  }
    }
    
    fn read(&self, adr: usize) -> u8{
        0
    }
    
    fn write(&self, adr: usize, data: u8){
        
    }
}
fn main() {
    println!("NES Emulator");
    
    let cartridge = Cartridge::new("games/mario.nes");
    let mut bus = Bus::new(&cartridge);
    
    let mut cpu = CPU6502::new(&mut bus);
    
    let debugMode = true;
    
    loop{
        cpu.tick();
        
        if debugMode{
            println!("{cpu}\n");
        }
        return;
    }
}
