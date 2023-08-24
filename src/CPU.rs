use std::fmt;
use crate::Bus_NES::*;

fn to16(h:u8,l:u8)->usize{
    ((h as usize)<<8)+l as usize
}
struct StatusRegister{
    N: u8,//bit 7
    V: u8,
    //unused
    B: u8,//weird wtf
    D: u8,
    I: u8,
    Z: u8,
    C: u8 //bit 0
}

//TODO: handle weird break flag logic here rather than in the instructions 
impl StatusRegister{
    fn new() -> StatusRegister{
        StatusRegister { N: 0, V: 0, B: 0, D: 0, I: 0, Z: 0, C: 0 }
    }
    
    fn asU8(&self) -> u8{
        //b flag is not sent here
        (self.N<<7)|(self.V<<6)|(1u8<<5)|(1u8<<4)|(self.D<<3)|(self.I<<2)|(self.Z<<1)|(self.C<<0)
    }
    
    fn fromU8(&mut self, data: u8){
        self.N = (data>>7)&1;
        self.V = (data>>6)&1;
        //unused bit
        //self.B //weird stuff i don't know
        self.D = (data>>3)&1;
        self.I = (data>>2)&1;
        self.Z = (data>>1)&1;
        self.C = (data>>0)&1;
    }
}

impl fmt::Display for StatusRegister{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
        let N = match self.N{1=>"N",_=>"-"};
        let V = match self.V{1=>"V",_=>"-"};
        let B = match self.B{1=>"B",_=>"-"};
        let D = match self.D{1=>"D",_=>"-"};
        let I = match self.I{1=>"I",_=>"-"};
        let Z = match self.Z{1=>"Z",_=>"-"};
        let C = match self.C{1=>"C",_=>"-"};
        write!(f,"{}{}-{}{}{}{}{}", N,V,B,D,I,Z,C)
    }
}
pub struct CPU6502<'a>{
    pc: usize,
    bus: &'a mut Bus,
    buffer:u8,
    acc: u8,
    x: u8,
    y: u8,
    status: StatusRegister,
    sp : u8,
    
    cycles: usize,
    opcode: String,
    adrMode: String,
    takeBranch: bool
}

impl fmt::Display for CPU6502<'_>{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"PC  ,ACC, X , Y, NV-BDIZC\n").unwrap();
        write!(f,"{} {} {} {} {} {} {}", self.pc, self.opcode, self.adrMode, self.acc, self.x, self.y, self.status)
    }
}

impl<'a> CPU6502<'a>{
    //type Instruction = fn (&mut Self)->();
    pub fn tick(&mut self){
        self.cycles = 0; //reset the cycles just in case
        let opcode = self.pcRead();
        
        match opcode{
            0x00 => self.Implied(Self::BRK),
            0x01 => self.ZeroPageIndexedXIndirect(Self::ORA),
            0x05 => self.ZeroPage(Self::ORA),
            0x06 => self.ZeroPage(Self::ASL),
            0x08 => self.Implied(Self::PHP),
            0x09 => self.Immediate(Self::ORA),
            0x0A => self.Accumulator(Self::ASL),
            0x0D => self.AbsoluteAdr(Self::ORA),
            0x0E => self.AbsoluteAdr(Self::ASL),
            0x10 => self.Relative(Self::BPL),
            0x11 => self.ZeroPageIndirectIndexedY(Self::ORA),
            0x15 => self.ZeroPageIndexedX(Self::ORA),
            0x16 => self.ZeroPageIndexedX(Self::ASL),
            0x18 => self.Implied(Self::CLC),
            0x19 => self.AbsoluteY(Self::ORA),
            0x1D => self.AbsoluteX(Self::ORA),
            0x1E => self.AbsoluteX(Self::ASL),
            0x20 => self.Implied(Self::JSR),
            0x21 => self.ZeroPageIndexedXIndirect(Self::AND),
            0x24 => self.ZeroPage(Self::BIT),
            0x25 => self.ZeroPage(Self::AND),
            0x26 => self.ZeroPageRMW(Self::ROL),
            0x28 => self.Implied(Self::PLP),
            0x29 => self.Immediate(Self::AND),
            0x2A => self.AccumulatorRMW(Self::ROL),
            0x2C => self.AbsoluteAdr(Self::BIT),
            0x2D => self.AbsoluteAdr(Self::AND),
            0x2E => self.AbsoluteRMW(Self::ROL),
            0x30 => self.Relative(Self::BMI),
            0x31 => self.ZeroPageIndirectIndexedY(Self::AND),
            0x35 => self.ZeroPageIndexedX(Self::AND),
            0x36 => self.ZeroPageIndexedXRMW(Self::ROL),
            0x38 => self.Implied(Self::SEC),
            0x39 => self.AbsoluteY(Self::AND),
            0x3D => self.AbsoluteX(Self::AND),
            0x3E => self.AbsoluteXRMW(Self::ROL),
            0x40 => self.Implied(Self::RTI),
            0x41 => self.ZeroPageIndexedXIndirect(Self::EOR),
            0x45 => self.ZeroPage(Self::EOR),
            0x46 => self.ZeroPageRMW(Self::LSR),
            0x48 => self.Implied(Self::PHA),
            0x49 => self.Immediate(Self::EOR),
            0x4A => self.AccumulatorRMW(Self::LSR),
            0x4C => self.Implied(Self::JMP_Absolute),
            0x4D => self.AbsoluteAdr(Self::EOR),
            0x4E => self.AbsoluteRMW(Self::LSR),
            0x50 => self.Relative(Self::BVC),
            0x51 => self.ZeroPageIndirectIndexedY(Self::EOR),
            0x55 => self.ZeroPageIndexedX(Self::EOR),
            0x56 => self.ZeroPageIndexedXRMW(Self::LSR),
            0x58 => self.Implied(Self::CLI),
            0x59 => self.AbsoluteY(Self::EOR),
            0x5D => self.AbsoluteX(Self::EOR),
            0x5E => self.AbsoluteXRMW(Self::LSR),
            0x60 => self.Implied(Self::RTS),
            0x61 => self.ZeroPageIndexedXIndirect(Self::ADC),
            0x65 => self.ZeroPage(Self::ADC),
            0x66 => self.ZeroPageRMW(Self::ROR),
            0x68 => self.Implied(Self::PLA),
            0x69 => self.Immediate(Self::ADC),
            0x6A => self.AccumulatorRMW(Self::ROR),
            0x6C => self.Implied(Self::JMP_Indirect),
            0x6D => self.AbsoluteAdr(Self::ADC),
            0x6E => self.AbsoluteRMW(Self::ROR),
            0x70 => self.Relative(Self::BVS),
            0x71 => self.ZeroPageIndirectIndexedY(Self::ADC),
            0x75 => self.ZeroPageIndexedX(Self::ADC),
            0x76 => self.ZeroPageIndexedXRMW(Self::ROR),
            0x78 => self.Implied(Self::SEI),
            0x79 => self.AbsoluteY(Self::ADC),
            0x7D => self.AbsoluteX(Self::ADC),
            0x7E => self.AbsoluteXRMW(Self::ROR),
            0x81 => self.ZeroPageIndexedXIndirectWrite(Self::STA),
            0x84 => self.ZeroPageWrite(Self::STY),
            0x85 => self.ZeroPageWrite(Self::STA),
            0x86 => self.ZeroPageWrite(Self::STX),
            0x88 => self.Implied(Self::DEY),
            0x8A => self.Implied(Self::TXA),
            0x8D => self.AbsoluteWrite(Self::STA),
            0x8E => self.AbsoluteWrite(Self::STX),
            0x8C => self.AbsoluteWrite(Self::STY),
            0x90 => self.Relative(Self::BCC),
            0x91 => self.ZeroPageIndirectIndexedYWrite(Self::STA),
            0x94 => self.ZeroPageIndexedXWrite(Self::STY),
            0x95 => self.ZeroPageIndexedXWrite(Self::STA),
            0x96 => self.ZeroPageIndexedYWrite(Self::STX),
            0x98 => self.Implied(Self::TYA),
            0x99 => self.AbsoluteYWrite(Self::STA),
            0x9A => self.Implied(Self::TXS),
            0x9D => self.AbsoluteXWrite(Self::STA),
            0xA0 => self.Immediate(Self::LDY),
            0xA1 => self.ZeroPageIndexedXIndirect(Self::LDA),
            0xA2 => self.Immediate(Self::LDX),
            0xA4 => self.ZeroPage(Self::LDY),
            0xA5 => self.ZeroPage(Self::LDA),
            0xA6 => self.ZeroPage(Self::LDX),
            0xA8 => self.Implied(Self::TAY),
            0xA9 => self.Immediate(Self::LDA),
            0xAA => self.Implied(Self::TAX),
            0xAC => self.AbsoluteAdr(Self::LDY),
            0xAD => self.AbsoluteAdr(Self::LDA),
            0xAE => self.AbsoluteAdr(Self::LDX),
            0xB0 => self.Relative(Self::BCS),
            0xB1 => self.ZeroPageIndirectIndexedY(Self::LDA),
            0xB4 => self.ZeroPageIndexedX(Self::LDY),
            0xB5 => self.ZeroPageIndexedX(Self::LDA),
            0xB6 => self.ZeroPageIndexedY(Self::LDX),
            0xB8 => self.Implied(Self::CLV),
            0xB9 => self.AbsoluteY(Self::LDA),
            0xBA => self.Implied(Self::TSX),
            0xBC => self.AbsoluteX(Self::LDY),
            0xBD => self.AbsoluteX(Self::LDA),
            0xBE => self.AbsoluteY(Self::LDX),
            0xC0 => self.Immediate(Self::CPY),
            0xC1 => self.ZeroPageIndexedXIndirect(Self::CMP),
            0xC4 => self.ZeroPage(Self::CPY),
            0xC5 => self.ZeroPage(Self::CMP),
            0xC6 => self.ZeroPageRMW(Self::DEC),
            0xC8 => self.Implied(Self::INY),
            0xC9 => self.Immediate(Self::CMP),
            0xCA => self.Implied(Self::DEX),
            0xCC => self.AbsoluteAdr(Self::CPY),
            0xCD => self.AbsoluteAdr(Self::CMP),
            0xCE => self.AbsoluteRMW(Self::DEC),
            0xD0 => self.Relative(Self::BNE),
            0xD1 => self.ZeroPageIndirectIndexedY(Self::CMP),
            0xD5 => self.ZeroPageIndexedX(Self::CMP),
            0xD6 => self.ZeroPageIndexedXRMW(Self::DEC),
            0xD8 => self.Implied(Self::CLD),
            0xD9 => self.AbsoluteY(Self::CMP),
            0xDD => self.AbsoluteX(Self::CMP),
            0xDE => self.AbsoluteXRMW(Self::DEC),
            0xE0 => self.Immediate(Self::CPX),
            0xE1 => self.ZeroPageIndexedXIndirect(Self::SBC),
            0xE4 => self.ZeroPage(Self::CPX),
            0xE5 => self.ZeroPage(Self::SBC),
            0xE6 => self.ZeroPageRMW(Self::INC),
            0xE8 => self.Implied(Self::INX),
            0xE9 => self.Immediate(Self::SBC),
            0xEA => self.Implied(Self::NOP),
            0xEC => self.AbsoluteAdr(Self::CPX),
            0xED => self.AbsoluteAdr(Self::SBC),
            0xEE => self.AbsoluteRMW(Self::INC),
            0xF0 => self.Relative(Self::BEQ),
            0xF1 => self.ZeroPageIndirectIndexedY(Self::SBC),
            0xF5 => self.ZeroPageIndexedX(Self::SBC),
            0xF6 => self.ZeroPageIndexedXRMW(Self::INC),
            0xF8 => self.Implied(Self::SED),
            0xF9 => self.AbsoluteY(Self::SBC),
            0xFD => self.AbsoluteX(Self::SBC),
            0xFE => self.AbsoluteXRMW(Self::INC),
            _=>()
        }
    }
    
    pub fn new(bus: &'a mut Bus)->Self{
        CPU6502 { 
            bus: bus,
            pc: 0,
            buffer:0,
            acc:0,
            x:0,
            y:0,
            status: StatusRegister::new(),
            sp: 0,
            cycles:0,
            opcode: "".to_owned(),
            adrMode: "".to_owned(),
            takeBranch: false
        }
    }
    
    fn updateNZFlags(&mut self, data:u8){
        self.status.N = data >> 7;
        self.status.Z = if data == 0 {1} else {0};
    }
    
    /*fn updateVFlag(&mut self, a:u8, b:u8, res:u8){
        
    }*/
    
    fn push(&mut self, data: u8){
        self.bus.write(to16(1, self.sp), data);
        self.sp-=1;
    }
    
    fn pop(&mut self) -> u8{
        self.sp+=1;
        self.bus.read(to16(1, self.sp))
    }
    
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
    
    fn BCC(&mut self){
        self.setOpcode("BCC");
        
        self.takeBranch = self.status.C == 0;
    }
    
    fn BCS(&mut self){
        self.setOpcode("BCS");
        
        self.takeBranch = self.status.C == 1;
    }
    
    fn BEQ(&mut self){
        self.setOpcode("BEQ");
        
        self.takeBranch = self.status.Z == 1;
    }
    
    fn BIT(&mut self){
        self.setOpcode("BIT");
        
        self.status.N = (self.buffer>>7)&1;
        self.status.V = (self.buffer>>6)&1;
        
        self.status.Z = if (self.acc & self.buffer)==0{1}else {0};
    }
    
    fn BMI(&mut self){
        self.setOpcode("BMI");
        
        self.takeBranch = self.status.N == 1;
    }
    
    fn BNE(&mut self){
        self.setOpcode("BNE");
        
        self.takeBranch = self.status.Z == 0;
    }
    
    fn BPL(&mut self){
        self.setOpcode("BPL");
        
        self.takeBranch = self.status.N == 0;
    }
    
    fn BRK(&mut self){
        self.setOpcode("BRK");
        self.cycles = 7;
        
        self.pc+=1; //ignores the param
        self.push((((self.pc)>>8)&255) as u8);
        self.push((self.pc&255) as u8);
        
        let reg = self.status.asU8();
        self.push(reg|(1u8<<4));//push with break flag set
        
        self.status.I = 1;
    }
    
    fn BVC(&mut self){
        self.setOpcode("BVC");
        
        self.takeBranch = self.status.V == 0;
    }
    
    fn BVS(&mut self){
        self.setOpcode("BVS");
        
        self.takeBranch = self.status.V == 1;
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
    
    fn CMP(&mut self){
        self.setOpcode("CMP");
        
        self.status.C = if self.acc >= self.buffer{1}else{0};
        let res = self.acc - self.buffer;
        self.updateNZFlags(res);
    }
    
    fn CPX(&mut self){
        self.setOpcode("CPX");
        
        self.status.C = if self.x >= self.buffer{1}else{0};
        let res = self.x - self.buffer;
        self.updateNZFlags(res);
    }
    
    fn CPY(&mut self){
        self.setOpcode("CPY");
        
        self.status.C = if self.y >= self.buffer{1}else{0};
        let res = self.y - self.buffer;
        self.updateNZFlags(res);
    }
    
    fn DEC(&mut self){
        self.setOpcode("DEC");
        
        self.buffer -=1;
        self.updateNZFlags(self.buffer);
    }
    
    fn DEX(&mut self){
        self.setOpcode("DEX");
        self.x -=1;
        
        self.updateNZFlags(self.x);
    }
    
    fn DEY(&mut self){
        self.setOpcode("DEY");
        self.y -=1;
        
        self.updateNZFlags(self.y);
    }
    
    fn EOR(&mut self){
        self.setOpcode("EOR");
        
        self.acc ^= self.buffer;
        
        self.updateNZFlags(self.acc);
    }
    
    fn INC(&mut self){
        self.setOpcode("INC");
        
        self.buffer +=1;
        self.updateNZFlags(self.buffer);
    }
    
    fn INX(&mut self){
        self.setOpcode("INX");
        self.x +=1;
        
        self.updateNZFlags(self.x);
    }
    
    fn INY(&mut self){
        self.setOpcode("INY");
        self.y +=1;
        
        self.updateNZFlags(self.y);
    }
    
    fn JMP_Absolute(&mut self){
        self.setOpcode("JMP");
        self.cycles = 3;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        self.pc = adr;
    }
    
    fn JMP_Indirect(&mut self){
        self.setOpcode("JMP");
        self.cycles = 5;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = "(".to_owned()+&adr.to_string()+")";
        
        let adrLow = self.bus.read(adr);
        let adrHigh = self.bus.read(adr+1);
        self.pc = to16(adrHigh, adrLow);
    }
    
    fn JSR(&mut self){
        self.setOpcode("JSR");
        self.cycles = 6;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        self.push(((self.pc>>8)&255) as u8);
        self.push((self.pc&255)as u8);
        
        self.pc = adr;
    }
    
    fn LDA(&mut self){
        self.setOpcode("LDA");
        
        self.acc = self.buffer;
        
        self.updateNZFlags(self.acc);
    }
    
    fn LDX(&mut self){
        self.setOpcode("LDX");
        
        self.x = self.buffer;
        
        self.updateNZFlags(self.x);
    }
    
    fn LDY(&mut self){
        self.setOpcode("LDY");
        
        self.y = self.buffer;
        
        self.updateNZFlags(self.y);
    }
    
    fn LSR(&mut self){
        self.setOpcode("LSR");
        
        self.status.C = self.buffer&1;
        self.buffer>>=1;
        
        self.updateNZFlags(self.buffer);
    }
    
    fn NOP(&mut self){
        self.setOpcode("NOP");
    }
    
    fn ORA(&mut self){
        self.setOpcode("ORA");
        
        self.acc |= self.buffer;
        self.updateNZFlags(self.acc);
    }
    
    fn PHA(&mut self){
        self.setOpcode("PHA");
        
        self.push(self.acc);
        
        self.cycles+=1;
    }
    
    fn PHP(&mut self){
        self.setOpcode("PHP");
        
        let status = self.status.asU8();// | (1u8<<5)|(1u8<<4);//set bit 5 and break flag to 1
        self.push(status);
        
        self.cycles+=1;
    }
    
    fn PLA(&mut self){
        self.setOpcode("PLA");
        
        self.acc = self.pop();
        self.updateNZFlags(self.acc);
        
        self.cycles+=2;
    }
    
    fn PLP(&mut self){
        self.setOpcode("PLP");
        
        let reg = self.pop();
        self.status.fromU8(reg);
        
        self.cycles+=2;
    }
    
    fn ROL(&mut self){
        self.setOpcode("ROL");
        
        let tmpC = self.status.C;
        self.status.C = (self.buffer>>7)&1;
        self.buffer<<=1;
        self.buffer|=tmpC;
        
        self.updateNZFlags(self.buffer);
    }
    
    fn ROR(&mut self){
        self.setOpcode("ROR");
        
        let tmpC = self.status.C;
        self.status.C = self.buffer&1;
        self.buffer>>=1;
        self.buffer|= tmpC<<7;
        
        self.updateNZFlags(self.buffer);
    }
    
    fn RTI(&mut self){
        self.setOpcode("RTI");
        self.cycles = 6;
        
        let reg = self.pop();
        self.status.fromU8(reg);
        
        let adrLow = self.pop();
        let adrHigh = self.pop();
        
        self.pc = to16(adrHigh, adrLow);
    }
    
    fn RTS(&mut self){
        self.setOpcode("RTS");
        self.cycles = 6;
        
        let adrLow = self.pop();
        let adrHigh = self.pop();
        
        self.pc = to16(adrHigh, adrLow);
        self.pc += 1;
    }
    
    fn SBC(&mut self){
        if self.status.D==1{
            //todo
        }
        else{
            //setReg(idb,~getReg(idb)); //1's complement (adding the carry will make it 2's complement, not adding it will make -idb-1(borrow)
            self.buffer = !self.buffer;
        }
        //self.acc = self.acc - self.buffer - (1-self.status.C);
        //self.updateNZFlags(self.acc);
        self.ADC();
        self.setOpcode("SBC");
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
        
        self.x = self.sp;
        self.updateNZFlags(self.sp);
    }
    
    fn TXA(&mut self){
        self.setOpcode("TXA");
        
        self.acc = self.x;
        self.updateNZFlags(self.acc);
    }
    
    fn TXS(&mut self){
        self.setOpcode("TXS");
        
        self.sp = self.x;
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
    
    /*fn handlePageCrossRelative(&mut self, adr:usize, data: u8)->usize{
        let adr = adr as u16;
        if ((adr&255)+data as u16)>>8 == 1{
            self.cycles+=1;
        }
        adr as usize+data as usize
    }*/
    
    fn AbsoluteAdr(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        self.buffer = self.bus.read(adr);
        
        inst(self);
    }
    
    fn AbsoluteRMW(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 6;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
        self.buffer = self.bus.read(adr);
        
        inst(self);
        
        self.bus.write(adr, self.buffer);
    }
    
    fn AbsoluteWrite(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 4;
        inst(self);
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let adr = to16(adrHigh, adrLow);
        
        self.adrMode = adr.to_string();
        
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
    
    fn AbsoluteXRMW(&mut self, inst:fn (&mut Self)->()){
        self.cycles = 7;
        
        let adrLow = self.pcRead();
        let adrHigh = self.pcRead();
        let baseAdr = to16(adrHigh, adrLow);
        
        self.adrMode = baseAdr.to_string()+",X";
        
        let adr = baseAdr + (self.x as usize);
        self.buffer = self.bus.read(adr);
        
        inst(self);
        
        self.bus.write(adr, self.buffer);
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
    
    fn AccumulatorRMW(&mut self, inst:fn(&mut Self)->()){
        self.cycles = 2; 
        self.adrMode = "".to_owned();
        
        self.buffer = self.acc;
        
        inst(self);
        
        self.acc = self.buffer;
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
    
    fn Relative(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 2;
        self.buffer = self.pcRead();
        
        self.adrMode = self.buffer.to_string();
        
        inst(self);
        
        if self.takeBranch{
            self.cycles+=1;
            self.takeBranch = false; //reset it for next use
            let high:usize = if (self.buffer>>7)&1 == 1{
                (255<<8) as usize
            } else{
                0
            };
            
            self.pc = self.handlePageCross(self.pc, self.buffer)+high;
        }
    }
    
    fn ZeroPage(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 3;
        
        let offset = self.pcRead();
        
        self.adrMode = offset.to_string();
        
        self.buffer = self.bus.read(to16(0, offset));
        
        inst(self);
    }
    
    fn ZeroPageRMW(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 5;
        
        let offset = self.pcRead();
        
        self.adrMode = offset.to_string();
        
        let adr = to16(0, offset);
        self.buffer = self.bus.read(adr);
        
        inst(self);
        
        self.bus.write(adr, self.buffer);
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
    
    fn ZeroPageIndexedXRMW(&mut self, inst: fn(&mut Self)->()){
        self.cycles = 6;
        
        let mut offset = self.pcRead();
        self.adrMode = offset.to_string()+",X";
        offset+=self.x;
        
        let adr = to16(0, offset);
        self.buffer = self.bus.read(adr);
        
        inst(self);
        
        self.bus.write(adr, self.buffer);
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