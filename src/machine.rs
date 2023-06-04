use crate::instruction::Instruction;

pub struct Machine {
    // 4kb of memory
    mem: [u8; 4096],
    // 16 8-bit registers
    // reg 15 is used as a flag
    regs: [u8; 16],
    // 16-bit address register
    i: u16,
    // 16-bit program counter
    pc: u16,
    // 8-bit stack pointer
    sp: u8,
    // 16 16-bit values
    stack: [u16; 16],
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            mem: [0; 4096],
            regs: [0; 16],
            i: 0,
            pc: 512,
            sp: 0,
            stack: [0; 16],
        }
    }

    // load a program into memory
    // programs start after the first page
    pub fn load(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.mem[i + 512] = byte;
        }
    }

    pub fn run(&mut self) {
        loop {
            self.dispatch();
        }
    }

    // 0x0000
    // return from subroutine
    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    // 0x1nnn
    // jump to address nnn
    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    // 0x2nnn
    // call subroutine at nnn
    fn call(&mut self, addr: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr;
    }

    // 0x3xkk
    // compare reg x to kk
    // skip next instruction if equal
    fn eq(&mut self, x: u8, byte: u8) {
        if self.regs[x as usize] == byte {
            self.pc += 2;
        }
    }

    // 0x4xkk
    // compare reg x to kk
    // skip next instruction if not equal
    fn neq(&mut self, x: u8, byte: u8) {
        if self.regs[x as usize] != byte {
            self.pc += 2;
        }
    }

    // 0x5xy0
    // compare reg x to reg y
    // skip next instruction if equal
    fn eq_xy(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] == self.regs[y as usize] {
            self.pc += 2;
        }
    }

    // 0x6xkk
    // set reg x to kk
    fn ld(&mut self, x: u8, byte: u8) {
        self.regs[x as usize] = byte;
    }

    // 0x7xkk
    // add kk to reg x
    // store result in reg x
    fn add(&mut self, x: u8, byte: u8) {
        self.regs[x as usize] = self.regs[x as usize].wrapping_add(byte);
    }

    // 0x8xy0
    // set reg x to reg y
    fn ld_xy(&mut self, x: u8, y: u8) {
        self.regs[x as usize] = self.regs[y as usize];
    }

    // 0x8xy1
    // set reg x to reg x | reg y
    fn or(&mut self, x: u8, y: u8) {
        self.regs[x as usize] |= self.regs[y as usize];
    }

    // 0x8xy2
    // set reg x to reg x & reg y
    fn and(&mut self, x: u8, y: u8) {
        self.regs[x as usize] &= self.regs[y as usize];
    }

    // 0x8xy3
    // set reg x to reg x ^ reg y
    fn xor(&mut self, x: u8, y: u8) {
        self.regs[x as usize] ^= self.regs[y as usize];
    }

    // 0x8xy4
    // add reg y to reg x
    // store result in reg x
    // set carry flag if result > 255
    fn add_xy(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[x as usize].overflowing_add(self.regs[y as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = overflow as u8;
    }

    // 0x8xy5
    // subtract reg y from reg x
    // store result in reg x
    // set borrow flag if reg y > reg x
    fn sub_xy(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[x as usize].overflowing_sub(self.regs[y as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = !overflow as u8;
    }

    // 0x8xy6
    // shift reg x right by 1
    // store result in reg x
    // set carry flag to least significant bit of reg x
    fn shr(&mut self, x: u8) {
        self.regs[0xF] = self.regs[x as usize] & 0x1;
        self.regs[x as usize] >>= 1;
    }

    // 0x8xy7
    // subtract reg x from reg y
    // store result in reg x
    // set borrow flag if reg x > reg y
    fn subn_xy(&mut self, x: u8, y: u8) {
        let (res, overflow) = self.regs[y as usize].overflowing_sub(self.regs[x as usize]);
        self.regs[x as usize] = res;
        self.regs[0xF] = !overflow as u8;
    }

    // 0x8xyE
    // shift reg x left by 1
    // store result in reg x
    // set carry flag to most significant bit of reg x
    fn shl(&mut self, x: u8) {
        self.regs[0xF] = self.regs[x as usize] >> 7;
        self.regs[x as usize] <<= 1;
    }

    // 0x9xy0
    // compare reg x to reg y
    // skip next instruction if not equal
    fn neq_xy(&mut self, x: u8, y: u8) {
        if self.regs[x as usize] != self.regs[y as usize] {
            self.pc += 2;
        }
    }

    // 0xAnnn
    // set index register to nnn
    fn ld_i(&mut self, addr: u16) {
        self.i = addr;
    }

    // 0xBnnn
    // jump to address nnn + reg 0
    fn jmp_v0(&mut self, addr: u16) {
        self.pc = addr + self.regs[0] as u16;
    }

    // 0xCx00
    // set index reg to I + reg x
    fn add_i(&mut self, x: u8) {
        self.i += self.regs[x as usize] as u16;
    }

    // 0xDx00
    // store binary coded decimal representation of reg x in memory
    // starting at address stored in index reg
    // 100s digit at index reg
    // 10s digit at index reg + 1
    // 1s digit at index reg + 2
    fn bcd(&mut self, x: u8) {
        let mut val = self.regs[x as usize];
        for i in (0..3).rev() {
            self.mem[(self.i + i as u16) as usize] = val % 10;
            val /= 10;
        }
    }

    // 0xEx00
    // store registers 0 through x in memory starting at reg I
    fn ld_0x(&mut self, x: u8) {
        for i in 0..x + 1 {
            self.mem[(self.i + i as u16) as usize] = self.regs[i as usize];
        }
    }

    // 0xFx00
    // store memory starting at reg I in registers 0 through x
    fn ld_x0(&mut self, x: u8) {
        for i in 0..x + 1 {
            self.regs[i as usize] = self.mem[(self.i + i as u16) as usize];
        }
    }

    // 0xFx01
    // prints registers 0 through x to screen
    fn out(&mut self, x: u8) {
        for i in 0..x + 1 {
            print!("{:X} ", self.regs[i as usize]);
        }
    }

    // 0xFx02
    // prints memory starting at reg I to screen
    // ends at reg I + x
    fn out_i(&mut self, x: u8) {
        for i in 0..x + 1 {
            print!("{:X} ", self.mem[(self.i + i as u16) as usize]);
        }
    }

    // 0xFFFF
    // exit program
    fn exit(&mut self) {
        println!("Exiting...");
        std::process::exit(1);
    }

    fn err(&mut self, instr: Instruction) {
        println!("Unknown instruction: {:X}", instr.0);
    }


    fn fetch(&mut self) -> Instruction {
        if self.pc >= self.mem.len() as u16 {
            panic!("PC out of bounds");
        }
        let instr = Instruction::from_bytes([self.mem[self.pc as usize], self.mem[(self.pc + 1) as usize]]);
        self.pc += 2;
        println!("{:X}", instr.0);
        instr
    }

    fn dispatch(&mut self) {
        let instr = self.fetch();
        match instr.opcode() {
            0x0 => self.ret(),
            0x1 => self.jmp(instr.nnn()),
            0x2 => self.call(instr.nnn()),
            0x3 => self.eq(instr.x(), instr.kk()),
            0x4 => self.neq(instr.x(), instr.kk()),
            0x5 => self.eq_xy(instr.x(), instr.y()),
            0x6 => self.ld(instr.x(), instr.kk()),
            0x7 => self.add(instr.x(), instr.kk()),
            0x8 => match instr.n() {
                0x0 => self.ld_xy(instr.x(), instr.y()),
                0x1 => self.or(instr.x(), instr.y()),
                0x2 => self.and(instr.x(), instr.y()),
                0x3 => self.xor(instr.x(), instr.y()),
                0x4 => self.add_xy(instr.x(), instr.y()),
                0x5 => self.sub_xy(instr.x(), instr.y()),
                0x6 => self.shr(instr.x()),
                0x7 => self.subn_xy(instr.x(), instr.y()),
                0xE => self.shl(instr.x()),
                _ => self.err(instr),
            },
            0x9 => self.neq_xy(instr.x(), instr.y()),
            0xA => self.ld_i(instr.nnn()),
            0xB => self.jmp_v0(instr.nnn()),
            0xC => self.add_i(instr.x()),
            0xD => self.bcd(instr.x()),
            0xE => self.ld_0x(instr.x()),
            0xF => match instr.n() {
                0x0 => self.ld_x0(instr.x()),
                0x1 => self.out(instr.x()),
                0x2 => self.out_i(instr.x()),
                0xF => self.exit(),
                _ => self.err(instr),
            },
            _ => self.err(instr),
        }
    }
}