#![allow(non_snake_case)]

mod CPU;
mod Bus_NES;
mod PPU_NES;

use crate::CPU::*;
use crate::Bus_NES::*;
use crate::PPU_NES::*;

use std::fs::File;
use std::io::Read;

pub trait Mapper{
    fn read(&self, adr: usize) -> u8;
    fn write(&mut self, adr: usize,  data: u8);
}

/*pub trait BinaryHandler{
    fn bit<T: From<bool>>(&self, bit:usize) -> T;
}
//idea: replace the complicated template with two different handlers: bit returns int, flag returns bool
impl BinaryHandler for u8{
    fn bit<T: From<bool>>(&self, bit: usize) -> T{
        T::from((self>>bit)&1==1)
    }
}*/

pub trait BinaryHandler{
    fn bit(&self, bit:usize) -> u8;
    fn flag(&self, bit:usize) -> bool;
    //fn bits(&self, bit:usize,size:usize) -> u8; //would return a packet of size bits
}
//idea: replace the complicated template with two different handlers: bit returns int, flag returns bool
impl BinaryHandler for u8{
    fn bit(&self, bit: usize) -> u8{
        (self>>bit)&1
    }
    fn flag(&self, bit:usize) -> bool {
        (self>>bit)&1 == 1
    }
}

pub struct Cartridge{
    prgROM: Vec<u8>,
    chrROM: Vec<u8>,
    ram: Vec<u8>,
}

impl Cartridge{
    fn new(path: &str) -> Cartridge {//Result<Cartridge, std::io::Error>{
        let mut file = File::open(path).expect("Can't find ROM file !");
        let mut ram = vec![0u8; 0x10000];
        file.read_exact(&mut ram).expect("failed to read rom!!");
        ram[0xFFFC] = 0x00;
        ram[0xFFFD] = 0x04;
        return Cartridge{ram: ram, prgROM:vec![0u8], chrROM: vec![0u8]};
        
        //let mut contents = Vec::new();
        //file.read_to_end(&mut contents).expect("Failed to read ROM file");
        //let bytes = file.bytes();//.take(16).collect();
        let mut header = vec![0u8;16];
        file.read_exact(&mut header).unwrap();
        
        //let header: Vec<u8> = bytes.take(16).map(|x|x.unwrap()).collect();
        
        if !(header[0] == b'N' && header[1] == b'E' && header[2] == b'S' && header[3] == 0x1A){
            //return Err("dsfsdfs");
            panic!("I don't handle other header formats than iNes");
        }
        
        //let var:u8 = contents[5].bit(4);
        
        let prgSize = (header[4] as usize)*16;//in kb (1024 bytes)
        let chrSize = (header[5] as usize)*8;
        println!("prgSize: {prgSize} chrSize: {chrSize}");
        println!("{} + {} = {}", prgSize*1024, chrSize*1024, (prgSize+chrSize)*1024);
        
        let mut PRG_ROM = vec![0u8; prgSize*1024];
        file.read_exact(&mut PRG_ROM).unwrap();
        
        let mut CHR_ROM = vec![0u8; chrSize*1024];
        file.read_exact(&mut CHR_ROM).unwrap();
        
        let flags6 = header[6];
        let flags7 = header[7];
        let flags8 = header[8];
        let flags9 = header[9];
        let flags10 = header[10];
        
        let mut nes2Format = false;
        if flags7 & 0x0C == 0x08{
            println!("NES 2.0 format !");
            panic!("format not handled yet !");
            nes2Format = true;
        }
        else {
            println!("NES 1.0 format !");
        }
        
        let mapper = (flags6 >> 4) | (flags7 & 0xF0);
        println!("Mapper: {}", mapper);
        
        if flags6.flag(0){
            println!("Horizontal arrangement (vertically mirrored)");
        }
        else{
            println!("Vertical arrangement (horizontally mirrored)");
        }
        
        if flags6.flag(1){
            println!("Contains Battery powered RAM");
        }
        
        if flags6.flag(2){
            println!("512 bytes trainer present !");
        }
        
        if flags6.flag(3){
            println!("Alternative nametable layout");
        }
        
        if flags7.flag(0){
            println!("VS Unisystem");
        }
        
        if flags7.flag(7){
            println!("Playchoice ");
        }
        
        let prgRAM = flags8;
        println!("PRG RAM: {}",prgRAM);
        
        if flags9.flag(0){
            println!("PAL system");
        }
        else {
            println!("NTSC system");
        }
        
        if flags10.flag(4){
            println!("No prgRAM");
        }
        
        if flags10.flag(5){
            println!("Board has bus conflicts");
        }
        
        if (header[11] != 0 || header[12] != 0 || header[13] != 0 || header[14] != 0 || header[15] != 0) && !nes2Format{
            panic!("not loading this shit");
        }
        
        /*if flags6.bit::<i32>(2) == 1{ //default is bool, this works but is ugly as fuck
            println!("test");
        }*/
        
        Cartridge { prgROM: PRG_ROM, chrROM: CHR_ROM , ram:vec![0u8]}
        /*if let Some(byte) = file.bytes().next(){
            println!("{}",byte?.bit::<bool>(5));
        }*/
        /* 
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).expect("Failed to read ROM file");
        //println!("{contents:?}");
        
        println!("header type: {}",contents.iter().take(3).map(|a| *a as char).collect::<String>());
        println!("{}",contents[1]);
        if contents[4].bit(2){
            println!("dcsdc");
        }*/
        //Ok(Cartridge {  })
    }
    
    //cartridge would own a mapper object of a certain empty type: Mapper0 that implements mapper => then the cartridge would be a generic type
    
    /*fn read(&self, adr: usize) -> u8{ 
        self.prgROM[adr - 0x8000]
    }*/
    
    /*fn write(&self, adr: usize, data: u8){
        
    }*/
}

impl Mapper for Cartridge{
    fn read(&self, adr: usize) -> u8{
        /*match adr{
            0x8000..=0xBFFF => self.prgROM[adr - 0x8000],
            0xC000..=0xFFFF => self.prgROM[adr - 0xC000],
            _ => {panic!("not handled {adr:#x}");}
        }*/
        return self.ram[adr];
    }
    fn write(&mut self, adr: usize,  data: u8){
        self.ram[adr] = data;
    }
}
fn main() {
    println!("NES Emulator");
    
    let mut cartridge = Cartridge::new("games/6502_functional_test.bin"); //nestest.nes");//.unwrap();
    
    let mut ppu = PPU::new();
    let mut bus = Bus::new(&mut cartridge, &mut ppu);
    
    let mut cpu = CPU6502::new(&mut bus);
    
    let debugMode = true;
    
    loop{
        cpu.tick();
        
        if debugMode{
            cpu.debugMode();
        }
        
        //return;
    }
}
