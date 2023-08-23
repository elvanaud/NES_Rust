const MEM_SIZE:usize = 1000;

pub struct Bus{
    memory: Vec<u8>
}

impl Bus{
    pub fn read(&self, adr: usize)->u8{
        self.memory[adr]
    }
    
    pub fn write(&mut self, adr:usize, data: u8){
        self.memory[adr] = data;
    }
    
    pub fn new()->Self{
        let mem = vec![0x69, 8, 0x69, 15, 0x65, 3];
        Bus{memory:mem}
    }
}