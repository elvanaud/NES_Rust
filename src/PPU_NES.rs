
use crate::Mapper;
pub struct PPU{
}

impl PPU{
    pub fn new() -> Self{
        PPU{ }
    }
}

impl Mapper for PPU{
    fn read(&self, adr: usize) -> u8{
        match adr{
            0x2002 => 0xFF,
            _ => {panic!("not handled {adr:#x}");}
        }
        
    }
    fn write(&mut self, adr: usize,  data: u8){
    }
}