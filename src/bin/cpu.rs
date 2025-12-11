use redis::{Commands, Connection};
use colored::Colorize;

struct Ram {
    conn: Connection,
}

impl Ram {
    fn new() -> Self {
        let client = redis::Client::open("redis://172.20.0.2/0").expect("Invalid Redis URL");
        Self { conn: client.get_connection().expect("Failed to connect") }
    }

    fn is_valid_hex(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c))
    }

    fn write(&mut self, addr_int: u16, data_int: u8) {

        let addr = format!("{:04X}", addr_int);
        let data = format!("{:02X}", data_int);

        assert_eq!(addr.len(), 4, "RAM_WRITE: Address must be 2 bytes. Got [{}]", addr);
        assert_eq!(data.len(), 2, "RAM_WRITE: Data must be 1 byte. Got [{}]", data);

        assert!(Self::is_valid_hex(&addr), "RAM_WRITE: Address must be 0-9, A-F");
        assert!(Self::is_valid_hex(&data), "RAM_WRITE: Data must be 0-9, A-F");

        let _: () = self.conn.set(format!("ram:{}", addr), data)
            .expect("Redis Write Failed");
    }

    fn read(&mut self, addr_int: u16) -> u8 {

        let addr = format!("{:04X}", addr_int);

        assert_eq!(addr.len(), 4, "RAM_READ: Address must be 2 bytes");
        assert!(Self::is_valid_hex(&addr), "RAM_READ: Address must be 0-9, A-F");

        let data: String = self.conn.get(format!("ram:{}", addr)).unwrap_or("00".to_string());

        assert_eq!(data.len(), 2, "RAM_READ: Data must be 1 byte. Got {}", data);
        assert!(Self::is_valid_hex(&data), "RAM_READ: Corrupt data [{}] in RAM (not 0-9, A-F)", data);

        u8::from_str_radix(&data, 16).unwrap()
    }
}

#[allow(non_snake_case)]
struct CPU {
    PC: u16,  // Program Counter
    R0: u8,   // Accumulator / Addr
    R1: u8,   // Data / Addr
    R2: u8,   // Stack Pointer
    R3: u8,   // Flags

    // -- internal vars --
    halted: bool,  // Halt Flag
    opcode: u8,    // Opcode
    
    // -- debug settings --
    fetch_debug: bool,   // Enable opcode output

    // -- ram access --
    ram: Ram,
}

impl CPU {
    fn new(ram: Ram) -> Self {
        Self {
            PC: 0x0000,
            R0: 0x00, R1: 0x00, R2: 0x00, R3: 0x00,

            // -- internal vars --
            halted: false,
            opcode: 0x00,
            
            // -- debug settings --
            fetch_debug: false,

            // -- ram access --
            ram,
        }
    }

    fn print_debug(&self) {
        println!("PC:[{:04X}], R0:[{:02X}], R1:[{:02X}], R2:[{:02X}], R3:[{:02X}]", self.PC, self.R0, self.R1, self.R2, self.R3);
    }

    fn fetch(&mut self) {
        self.opcode = self.ram.read(self.PC);
        self.PC += 1;

        if self.fetch_debug {
            println!("{}", format!("opcode: {:02X}", self.opcode).blue());
        }
    }

    fn execute(&mut self) {
        if self.opcode == 0x00 {

        } else if self.opcode == 0xF0 {
            self.halted = true;

        } else {
            panic!("{}", format!("CPU: Unknown opcode [{}]", self.opcode).red().bold());
        }
    }
}

fn preload_ram(ram: &mut Ram) {
    ram.write(0x0000, 0x00);   // NOP
    ram.write(0x0001, 0x00);   // NOP
    ram.write(0x0002, 0xF0);   // HALT
}

fn main() {
    let mut ram = Ram::new();
    preload_ram(&mut ram);

    let mut cpu = CPU::new(ram);

    println!("-- CPU STARTED ---------------------------------------------------");

    cpu.print_debug();
    cpu.fetch_debug = true;

    while !cpu.halted {
        cpu.fetch();
        cpu.execute();
        cpu.print_debug();
    }

    println!("{}", "CPU halted.".magenta());
    println!("-- CPU ENDED -----------------------------------------------------");
    
}

