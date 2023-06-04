// Instructions are 16 bits long
// The first 4 bits are the opcode
// The remaining 12 bits are the arguments
// A nibble or (n) is 4 bits
// nnn is a 12 bit value at 0x0fff
// x and y are 4 bit values  at 0x0f00 and 0x00f0 respectively
// kk is an 8 bit value at 0x00ff
// n is a 4 bit value at 0x000f
// E means the opcode takes no arguments
// I could have done this as a type alias
// but rust sees this struct as a u16
// and I can have helper methods
#[derive(Debug, Copy, Clone)]
pub struct Instruction(pub u16);


impl Instruction {
    pub fn opcode(&self) -> u16 {
        self.0 >> 12
    }

    pub fn nnn(&self) -> u16 {
        self.0 & 0x0fff
    }

    pub fn x(&self) -> u8 {
        ((self.0 & 0x0f00) >> 8) as u8
    }

    pub fn y(&self) -> u8 {
        ((self.0 & 0x00f0) >> 4) as u8
    }

    pub fn kk(&self) -> u8 {
        (self.0 & 0x00ff) as u8
    }

    pub fn n(&self) -> u8 {
        (self.0 & 0x000f) as u8
    }

    pub fn to_string(&self) -> String {
        format!("{:04x}", self.0)
    }

    pub fn as_bytes(self) -> [u8; 2] {
        [(self.0 >> 8) as u8, (self.0 & 0xff) as u8]
    }

    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Instruction(((bytes[0] as u16) << 8) | bytes[1] as u16)
    }
}