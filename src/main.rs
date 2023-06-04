mod instruction;
mod machine;

fn main() {
    let mut machine = machine::Machine::new();
    machine.load(&[0x60, 0x11,
                            0x61, 0x22,
                            0x80, 0x14,   
                            0xF0, 0x01,
                            0xFF, 0xFF]);
    machine.run();
}
