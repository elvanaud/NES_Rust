#![allow(non_snake_case)]

mod CPU;
mod Bus_NES;

use crate::CPU::*;
use crate::Bus_NES::*;


fn main() {
    println!("NES Emulator");
    
    
    let mut bus = Bus::new();
    
    let mut cpu = CPU6502::new(&mut bus);
    
    let debugMode = true;
    
    loop{
        cpu.tick();
        
        if debugMode{
            println!("Debug mode !\n{cpu}");
        }
    }
}
