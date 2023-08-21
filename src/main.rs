#![allow(non_snake_case)]

const MEM_SIZE:usize = 1000;

fn to16(h:u8,l:u8)->usize{
    (h as usize)<<8+l
}
struct Bus{
    memory: Vec<u8>
}

impl Bus{
    fn read(&self, adr: usize)->u8{
        self.memory[adr]
    }
    
    fn write(&mut self, adr:usize, data: u8){
        self.memory[adr] = data;
    }
    
    fn new()->Self{
        Bus{memory:vec![0u8; MEM_SIZE]}
    }
}
struct CPU6502<'a>{
    pc: usize,
    bus: &'a mut Bus,
    buffer:u8,
    acc: u8,
    x: u8,
    y: u8,
    
    cycles:usize
}

impl<'a> CPU6502<'a>{
    //type Instruction = fn (&mut Self)->();
    fn tick(&mut self){
        self.cycles = 0; //reset the cycles just in case
        let opcode = self.pcRead();
        
        match opcode{
            0x61 => self.ZeroPageIndexedIndirect(Self::ADC),
            0x65 => self.ZeroPage(Self::ADC),
            0x69 => self.Immediate(Self::ADC),
            0x6D => self.AbsoluteAdr(Self::ADC),
            0x71 => self.ZeroPageIndirectIndexed(Self::ADC),
            0x75 => self.ZeroPageIndexedX(Self::ADC),
            _=>()
        }
    }
    
    fn new(bus: &'a mut Bus)->Self{
        CPU6502 { 
            bus: bus,
            pc: 0,
            buffer:0,
            acc:0,
            x:0,
            y:0,
            cycles:0
        }
    }
    
    fn ADC(&mut self){
        
        self.acc+=self.buffer;
    }
    
    fn pcRead(&mut self)->u8{
        let data = self.bus.read(self.pc);
        self.pc+=1;
        data
    }
    
    fn handlePageCross(&mut self, adr:usize, data: u8)->usize{
        let adr = adr as u16;
        if ((adr&255)+data as u16)>>8 == 1{
            self.cycles+=1;
        }
        adr as usize+data as usize
    }
    
    fn AbsoluteAdr(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        
        self.buffer = self.bus.read(to16(adrHigh, adrLow));
        
        inst(self);
    }
    
    fn Immediate(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 2;
        
        self.buffer = self.pcRead();
        
        inst(self);
    }
    
    fn ZeroPage(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 3;
        
        let offset = self.pcRead();
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    fn ZeroPageIndexedIndirect(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 6;
        
        let offset = self.pcRead() + self.x;
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        self.buffer = self.bus.read(to16(adrHigh, adrLow));
        
        inst(self);
    }
    
    fn ZeroPageIndirectIndexed(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 5;
        
        let offset = self.pcRead();
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        let adr = self.handlePageCross(to16(adrHigh, adrLow),self.y);
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn ZeroPageIndexedX(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 4;
        
        let offset = self.pcRead() + self.x;
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    /*fn read(adr: usize){
        
    }*/
}


fn main() {
    println!("NES Emulator");
    
    
    let mut bus = Bus::new();
    
    let mut cpu = CPU6502::new(&mut bus);
    
    loop{
        cpu.tick();
    }
}
