const MEM_SIZE:usize = 0x800;
use crate::Cartridge;

pub struct Bus<'a>{
    memory: Vec<u8>,
    cart: &'a Cartridge
}

impl<'a> Bus<'a>{
    pub fn read(&self, adr: usize)->u8{
        match adr as u16{
            0x0000..=0x07FF => self.memory[adr-0x0000], //RAM
            0x0800..=0x0FFF => self.memory[adr-0x0800], //Mirrors
            0x1000..=0x17FF => self.memory[adr-0x1000],
            0x1800..=0x1FFF => self.memory[adr-0x1800],
            0x2000..=0x3FFF => 0, //PPU registers and mirrors
            0x4000..=0x401F => 0, //APU and IO stuff
            0x4020..=0xFFFF => self.cart.read(adr-0x4020), //Cartridge space
        }
    }
    
    pub fn write(&mut self, adr:usize, data: u8){
        self.memory[adr] = data;
        match adr as u16{
            0x0000..=0x07FF => self.memory[adr-0x0000] = data,        //RAM
            0x0800..=0x0FFF => self.memory[adr-0x0800] = data, //Mirrors
            0x1000..=0x17FF => self.memory[adr-0x1000] = data,
            0x1800..=0x1FFF => self.memory[adr-0x1800] = data,
            0x2000..=0x3FFF => (), //PPU registers and mirrors
            0x4000..=0x401F => (), //APU and IO stuff
            0x4020..=0xFFFF => self.cart.write(adr-0x4020, data), //Cartridge space
        }
    }
    
    pub fn new(cart: &'a Cartridge)->Self{
        //let mem = vec![0x69, 8, 0x69, 15, 0x65, 3];
        let mem = vec![0; MEM_SIZE];
        //mem[0xFFFC] = 0x00;
        //mem[0xFFFD] = 0x0C;
        Bus{
            memory:mem,
            cart: cart
        }
    }
}