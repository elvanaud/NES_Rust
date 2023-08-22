#![allow(non_snake_case)]

use core::fmt;
use std::fmt::Formatter;

const MEM_SIZE:usize = 1000;

fn to16(h:u8,l:u8)->usize{
    ((h as usize)<<8)+l as usize
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
        let mem = vec![0x69, 8, 0x69, 15, 0x65, 3];
        Bus{memory:mem}
    }
}

struct StatusRegister{
    N: u8,
    Z: u8,
    C: u8,
    I: u8,
    D: u8,
    V: u8
}

impl StatusRegister{
    fn new() -> StatusRegister{
        StatusRegister { N: 0, Z: 0, C: 0, I: 0, D: 0, V: 0 }
    }
}
struct CPU6502<'a>{
    pc: usize,
    bus: &'a mut Bus,
    buffer:u8,
    acc: u8,
    x: u8,
    y: u8,
    status: StatusRegister,
    s : u8,
    
    cycles: usize,
    opcode: String,
    adrMode: String
}

impl fmt::Display for CPU6502<'_>{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"PC  ,ACC, X , Y, NZCIDV\n").unwrap();
        write!(f,"{} {} {} {} {} {}", self.pc, self.opcode, self.adrMode, self.acc, self.x, self.y)
    }
}

impl<'a> CPU6502<'a>{
    //type Instruction = fn (&mut Self)->();
    fn tick(&mut self){
        self.cycles = 0; //reset the cycles just in case
        let opcode = self.pcRead();
        
        match opcode{
            0x00 => (),
            0x06 => self.ZeroPage(Self::ASL),
            0x0A => self.Accumulator(Self::ASL),
            0x0E => self.AbsoluteAdr(Self::ASL),
            0x16 => self.ZeroPageIndexedX(Self::ASL),
            0x18 => self.Implied(Self::CLC),
            0x1E => self.AbsoluteX(Self::ASL),
            0x21 => self.ZeroPageIndexedXIndirect(Self::AND),
            0x25 => self.ZeroPage(Self::AND),
            0x29 => self.Immediate(Self::AND),
            0x2D => self.AbsoluteAdr(Self::AND),
            0x31 => self.ZeroPageIndirectIndexedY(Self::AND),
            0x35 => self.ZeroPageIndexedX(Self::AND),
            0x38 => self.Implied(Self::SEC),
            0x39 => self.AbsoluteY(Self::AND),
            0x3D => self.AbsoluteX(Self::AND),
            0x58 => self.Implied(Self::CLI),
            0x61 => self.ZeroPageIndexedXIndirect(Self::ADC),
            0x65 => self.ZeroPage(Self::ADC),
            0x69 => self.Immediate(Self::ADC),
            0x6D => self.AbsoluteAdr(Self::ADC),
            0x71 => self.ZeroPageIndirectIndexedY(Self::ADC),
            0x75 => self.ZeroPageIndexedX(Self::ADC),
            0x7D => self.AbsoluteX(Self::ADC),
            0x78 => self.Implied(Self::SEI),
            0x79 => self.AbsoluteY(Self::ADC),
            0x81 => self.ZeroPageIndexedXIndirectWrite(Self::STA),
            0x84 => self.ZeroPageWrite(Self::STY),
            0x85 => self.ZeroPageWrite(Self::STA),
            0x86 => self.ZeroPageWrite(Self::STX),
            0x88 => self.Implied(Self::CLV),
            0x8A => self.Implied(Self::TXA),
            0x8D => self.AbsoluteWrite(Self::STA),
            0x8E => self.AbsoluteWrite(Self::STX),
            0x8C => self.AbsoluteWrite(Self::STY),
            0x91 => self.ZeroPageIndirectIndexedYWrite(Self::STA),
            0x94 => self.ZeroPageIndexedXWrite(Self::STY),
            0x95 => self.ZeroPageIndexedXWrite(Self::STA),
            0x96 => self.ZeroPageIndexedYWrite(Self::STX),
            0x98 => self.Implied(Self::TYA),
            0x99 => self.AbsoluteYWrite(Self::STA),
            0x9A => self.Implied(Self::TXS),
            0x9D => self.AbsoluteXWrite(Self::STA),
            0xAA => self.Implied(Self::TAX),
            0xA8 => self.Implied(Self::TAY),
            0xBA => self.Implied(Self::TSX),
            0xD8 => self.Implied(Self::CLD),
            0xF8 => self.Implied(Self::SED),
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
            status: StatusRegister::new(),
            s: 0,
            cycles:0,
            opcode: "".to_owned(),
            adrMode: "".to_owned()
        }
    }
    
    fn updateNZFlags(&mut self, data:u8){
        self.status.N = data >> 7;
        self.status.Z = if data == 0 {1} else {0};
    }
    
    /*fn updateVFlag(&mut self, a:u8, b:u8, res:u8){
        
    }*/
    
    fn setOpcode(&mut self, op: &str){
        self.opcode = op.to_owned()
    }
    
    fn ADC(&mut self){
        self.setOpcode("ADC");
        let data = self.acc as usize +self.buffer as usize+self.status.C as usize;
        self.status.C = (data>>8) as u8;
        
        let signA = self.acc>>7;
        let signB = self.buffer>>7;
        let signRes = ((data>>7)&1) as u8;
        
        if signA == signB{
            self.status.V = (signA != signRes) as u8;
        }
        self.acc = data as u8;
        self.updateNZFlags(self.acc);
        
    }
    
    fn AND(&mut self){
        self.setOpcode("AND");
        self.acc&=self.buffer;
        self.updateNZFlags(self.acc);
    }
    
    fn ASL(&mut self){
        self.setOpcode("ASL");
        let c = self.buffer>>7;
        self.status.C = c;
        self.acc<<=1;
        self.updateNZFlags(self.acc);
        self.cycles+=2;
    }
    
    fn CLC(&mut self){
        self.setOpcode("CLC");
        
        self.status.C = 0;
    }
    
    fn CLD(&mut self){
        self.setOpcode("CLD");
        
        self.status.D = 0;
    }
    
    fn CLI(&mut self){
        self.setOpcode("CLI");
        
        self.status.I = 0;
    }
    
    fn CLV(&mut self){
        self.setOpcode("CLV");
        
        self.status.V = 0;
    }
    
    fn SEC(&mut self){
        self.setOpcode("SEC");
        
        self.status.C = 1;
    }
    
    fn SED(&mut self){
        self.setOpcode("SED");
        
        self.status.D = 1;
    }
    
    fn SEI(&mut self){
        self.setOpcode("SEI");
        
        self.status.I = 1;
    }
    
    fn STA(&mut self){
        self.setOpcode("STA");
        
        self.buffer = self.acc;
    }
    
    fn STX(&mut self){
        self.setOpcode("STX");
        
        self.buffer = self.x;
    }
    
    fn STY(&mut self){
        self.setOpcode("STY");
        
        self.buffer = self.y;
    }
    
    fn TAX(&mut self){
        self.setOpcode("TAX");
        
        self.x = self.acc;
        self.updateNZFlags(self.acc);
    }
    
    fn TAY(&mut self){
        self.setOpcode("TAY");
        
        self.y = self.acc;
        self.updateNZFlags(self.acc);
    }
    
    fn TSX(&mut self){
        self.setOpcode("TSX");
        
        self.x = self.s;
        self.updateNZFlags(self.s);
    }
    
    fn TXA(&mut self){
        self.setOpcode("TXA");
        
        self.acc = self.x;
        self.updateNZFlags(self.acc);
    }
    
    fn TXS(&mut self){
        self.setOpcode("TXS");
        
        self.s = self.x;
        self.updateNZFlags(self.x);
    }
    
    fn TYA(&mut self){
        self.setOpcode("TYA");
        
        self.acc = self.y;
        self.updateNZFlags(self.acc);
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
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn AbsoluteWrite(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        inst(self);
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        //self.buffer = self.bus.read(adr);
        self.bus.write(adr, self.buffer);
        
    }
    
    fn AbsoluteX(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let baseAdr = to16(adrHigh, adrLow);
        
        self.adrMode = baseAdr.to_string()+",X";
        
        let adr = self.handlePageCross(baseAdr,self.x);
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn AbsoluteXWrite(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 5;
        inst(self);
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let baseAdr = to16(adrHigh, adrLow);
        
        self.adrMode = baseAdr.to_string()+",X";
        
        let adr = baseAdr + (self.x as usize);
        self.bus.write(adr, self.buffer);
    }
    
    fn AbsoluteY(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let baseAdr = to16(adrHigh, adrLow);
        
        self.adrMode = baseAdr.to_string()+",Y";
        
        let adr = self.handlePageCross(baseAdr,self.y);
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn AbsoluteYWrite(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 5;
        inst(self);
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let baseAdr = to16(adrHigh, adrLow);
        
        self.adrMode = baseAdr.to_string()+",Y";
        
        let adr = baseAdr + (self.y as usize);
        self.bus.write(adr, self.buffer);
    }
    
    fn Accumulator(&mut self, inst:fn(&mut Self)->()){
        self.cycles = 0; //always paired with instruction that add 2 cycles to the other modes
        self.adrMode = "".to_owned();
        
        self.buffer = self.acc;
        
        inst(self);
    }
    
    fn Immediate(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 2;
        
        self.buffer = self.pcRead();
        
        self.adrMode = "#".to_owned()+&self.buffer.to_string();
        
        inst(self);
    }
    
    fn Implied(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 2;
        
        self.adrMode = "".to_owned();
        
        inst(self);
    }
    
    fn ZeroPage(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 3;
        
        let offset = self.pcRead();
        
        self.adrMode = offset.to_string();
        
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    fn ZeroPageWrite(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 3;
        inst(self);
        
        let offset = self.pcRead();
        
        self.adrMode = offset.to_string();
        
        self.bus.write(to16(0, offset), self.buffer);
        
    }
    
    fn ZeroPageIndexedXIndirect(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 6;
        
        let mut offset = self.pcRead();
        self.adrMode = "(".to_owned()+&offset.to_string()+",X)";
        offset += self.x;
        
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        self.buffer = self.bus.read(to16(adrHigh, adrLow));
        
        inst(self);
    }
    
    fn ZeroPageIndexedXIndirectWrite(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 6;
        inst(self);
        
        let mut offset = self.pcRead();
        self.adrMode = "(".to_owned()+&offset.to_string()+",X)";
        offset += self.x;
        
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        
        self.bus.write(to16(adrHigh, adrLow),self.buffer);
    }
    
    fn ZeroPageIndirectIndexedY(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 5;
        
        let offset = self.pcRead();
        self.adrMode = "(".to_owned()+&offset.to_string()+"),Y";
        
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        let adr = self.handlePageCross(to16(adrHigh, adrLow),self.y);
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn ZeroPageIndirectIndexedYWrite(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 6;
        inst(self);
        
        let offset = self.pcRead();
        self.adrMode = "(".to_owned()+&offset.to_string()+"),Y";
        
        let adrLow = self.bus.read(to16(0, offset));
        let adrHigh = self.bus.read(to16(0, offset+1));
        let adr = to16(adrHigh, adrLow)+(self.y as usize);
        self.bus.write(adr, self.buffer);
    }
    
    fn ZeroPageIndexedX(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 4;
        
        let mut offset = self.pcRead();
        self.adrMode = offset.to_string()+",X";
        offset+=self.x;
        
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    fn ZeroPageIndexedXWrite(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 4;
        inst(self);
        
        let mut offset = self.pcRead();
        self.adrMode = offset.to_string()+",X";
        offset+=self.x;
        
        self.bus.write(to16(0,offset), self.buffer);
    }
    
    fn ZeroPageIndexedY(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 4;
        
        let mut offset = self.pcRead();
        self.adrMode = offset.to_string()+",Y";
        offset+=self.y;
        
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    fn ZeroPageIndexedYWrite(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 4;
        inst(self);
        
        let mut offset = self.pcRead();
        self.adrMode = offset.to_string()+",Y";
        offset+=self.y;
        
        self.bus.write(to16(0,offset), self.buffer);
    }
    
    /*fn read(adr: usize){
        
    }*/
}


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
