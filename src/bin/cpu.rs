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
    PC: u16,    // Program Counter
    R0: u8,     // General Purpose
    R1: u8,     // General Purpose
    R2: u8,     // General Purpose
    R3: u8,     // General Purpose
    SP: u16,    // Stack Pointer
    // -- flags --
    Z: bool,    // Zero
    N: bool,    // Negative
    C: bool,    // Carry / Unsigned Overflow
    V: bool,    // Signed Overflow

    // -- internal vars --
    halted: bool,  // Halt Flag
    instr:  u8,    // Instruction
    opcode: u8,    // Opcode (4bit)
    arg:    u8,    // Argument (4bit)
    
    // -- debug settings --
    fetch_debug: bool,   // Enable instruction output

    // -- ram access --
    ram: Ram,
}

const ZF: u8 = 1 << 7;
const NF: u8 = 1 << 6;
const CF: u8 = 1 << 5;
const VF: u8 = 1 << 4;

impl CPU {
    fn new(ram: Ram) -> Self {
        Self {
            PC: 0x0000,
            R0: 0x00, R1: 0x00, R2: 0x00, R3: 0x00,
            SP: 0x0000,
            Z: false, N: false, C: false, V: false,

            // -- internal vars --
            halted: false,
            instr:  0x00,
            opcode: 0x00,
            arg:    0x00,

            // -- debug settings --
            fetch_debug: false,

            // -- ram access --
            ram,
        }
    }

    fn sr(&self) -> u8 {
        let mut sr: u8 = 0;
        if self.Z { sr = sr | ZF }
        if self.N { sr = sr | NF }
        if self.C { sr = sr | CF }
        if self.V { sr = sr | VF }
        sr
    }

    fn set_rs(&mut self, sr: u8) {
        self.Z = (sr & ZF) != 0;
        self.N = (sr & NF) != 0;
        self.C = (sr & CF) != 0;
        self.V = (sr & VF) != 0;
    }

    fn flags_str(&self) -> String {
        format!(
            "{}{}{}{}",
            if self.Z { 'Z' } else { '-' },
            if self.N { 'N' } else { '-' },
            if self.C { 'C' } else { '-' },
            if self.V { 'V' } else { '-' },
        )
    }

    fn print_debug(&self) {
        println!("PC:[{:04X}], R0:[{:02X}], R1:[{:02X}], R2:[{:02X}], R3:[{:02X}], SP: [{:04X}], SR:[{}]",
            self.PC, self.R0, self.R1, self.R2, self.R3, self.SP, self.flags_str());
    }

    fn fetch(&mut self) {
        self.instr = self.ram.read(self.PC);
        self.incpc();

        if self.fetch_debug {
            println!("{}", format!("instr: {:02X}", self.instr).blue());
        }
    }

    fn incpc(&mut self) {
        self.PC += 1;
    }

    fn decpc(&mut self) {
        self.PC -= 1;
    }

    fn alu_add(&mut self, a: u8, b: u8) -> u8 {
        let tmp = a as u16 + b as u16;
        let r = (tmp & 0xFF) as u8;

        self.Z = r == 0;
        self.N = (r & 0x80) != 0;
        self.C = tmp > 0xFF;
        self.V = (!(a ^ b) & (a ^ r) & 0x80) != 0;

        r
    }

    fn alu_adc(&mut self, a: u8, b: u8) -> u8 {
        let c :u16 = if self.C { 1 } else { 0 };
        let tmp = a as u16 + b as u16 + c;
        let r = (tmp & 0xFF) as u8;

        self.Z = r == 0;
        self.N = (r & 0x80) != 0;
        self.C = tmp > 0xFF;
        self.V = (!(a ^ b) & (a ^ r) & 0x80) != 0;

        r
    }

    fn execute(&mut self) {
        self.opcode = self.instr >> 4;
        self.arg = self.instr & 0x0F;  

        match self.opcode {
            0x0 => {},                                  // NOP

            0x1 => {                                    // MOV Rx, [PC]
                self.fetch();

                match self.arg {
                    0x0 => self.R0 = self.instr,        // MOV R0, [PC]
                    0x1 => self.R1 = self.instr,        // MOV R1, [PC]
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }
            },

            0x2 => {                                    // MOV Rx, [R1 R0]
match self.arg {
                    0x0 => self.R0 = self.ram.read((self.R3 as u16) << 4 | self.R2 as u16),     // MOV R0, [R3 R2]
                    0x1 => self.R1 = self.ram.read((self.R3 as u16) << 4 | self.R2 as u16),     // MOV R1, [R3 R2]
                    0x2 => self.R2 = self.ram.read((self.R1 as u16) << 4 | self.R0 as u16),     // MOV R2, [R1 R0]
                    0x3 => self.R3 = self.ram.read((self.R1 as u16) << 4 | self.R0 as u16),     // MOV R3, [R1 R0]
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }
            },

            0x3 => {                                    // MOV [R1 R0], Rx
                match self.arg {
                    0x1 => self.ram.write((self.R3 as u16) << 4 | self.R2 as u16, self.R0),     // MOV [R3 R2], R0
                    0x2 => self.ram.write((self.R3 as u16) << 4 | self.R2 as u16, self.R1),     // MOV [R3 R2], R1
                    0x2 => self.ram.write((self.R1 as u16) << 4 | self.R0 as u16, self.R2),     // MOV [R1 R0], R2
                    0x3 => self.ram.write((self.R1 as u16) << 4 | self.R0 as u16, self.R3),     // MOV [R1 R0], R3
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }
            },

            0x4 => {                                    // ALU R0, R1
                match self.arg {
                    0x0 => self.R0 = self.alu_add(self.R0, self.R2),    // ADD R0, R2
                    0x1 => self.R1 = self.alu_add(self.R1, self.R3),    // ADD R1, R3
                    0x2 => self.R0 = self.alu_adc(self.R0, self.R2),    // ADC R0, R2
                    0x3 => self.R1 = self.alu_adc(self.R1, self.R3),    // ADC R1, R3
                    // 0x4                                              // NOT R0
                    // 0x5                                              // NOT R1
                    // 0x6                                              // AND R0, R2
                    // 0x7                                              // AND R1, R3
                    // 0x8                                              // XOR R0, R2
                    // 0x9                                              // XOR R1, R3
                    // 0xA                                              // SHR R0
                    // 0xB                                              // SHR R1
                    // 0xC                                              // SHL R0
                    // 0xD                                              // SHL R1
                    // 0xE                                              // CMP R0, R2
                    // 0xF                                              // CMP R1, R3
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }

            },
            0x5 => {},                                  // More math

            0x6 => {                                    // MOV R0/R1, Rx
                match self.arg {
                    0x1 => self.R0 = self.R1,           // MOV R0, R1
                    0x2 => self.R0 = self.R2,           // MOV R0, R2
                    0x3 => self.R0 = self.R3,           // MOV R0, R3
                    0x8 => self.R1 = self.R0,           // MOV R1, R0
                    0xA => self.R1 = self.R2,           // MOV R1, R2
                    0xB => self.R1 = self.R3,           // MOV R1, R3
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }
            },
            
            0x7 => {                                    // MOV Rx, R0/R1
                match self.arg {
                    0x1 => self.R1 = self.R0,           // MOV R1, R0
                    0x2 => self.R2 = self.R0,           // MOV R2, R0
                    0x3 => self.R3 = self.R0,           // MOV R3, R0
                    0x8 => self.R0 = self.R1,           // MOV R0, R1
                    0xA => self.R2 = self.R1,           // MOV R2, R1
                    0xB => self.R3 = self.R1,           // MOV R3, R1
                    _ => panic!("{}", format!("CPU: Unknown arg [{:X}]", self.arg).red().bold()),
                }
            },

            0x8 => {},                              // INC/DEC Rx

            0xF => {                                // HALT
                self.halted = true;
            },

            _ => panic!("{}", format!("CPU: Unknown instr [{:02X}]", self.instr).red().bold())
        }
    }
}

fn preload_ram(ram: &mut Ram) -> std::io::Result<()> {
    let program_hex = std::fs::read_to_string("program.hex")?;
    let mut addr: u16 = 0;

    for line in program_hex.lines() {
        let line_vec: Vec<_> = line.split("#").collect();
        let line_hex = line_vec[0].trim();
        let hex_vec: Vec<_> = line_hex.split(" ").collect();

        for hex in hex_vec.iter() {
            let val  = u8::from_str_radix(&hex, 16).unwrap();
            println!("hex: [{:04X}]: {:02X}", addr, val);
            ram.write(addr, val);
            addr += 1;
        }
    }

    Ok(())
}

fn main() {
    let mut ram = Ram::new();
    preload_ram(&mut ram);

    let mut cpu = CPU::new(ram);

    println!("-- CPU STARTED -----------------------------------------------------------------------------");

    cpu.print_debug();
    cpu.fetch_debug = true;

    while !cpu.halted {
        cpu.fetch();
        cpu.execute();
        cpu.print_debug();
    }

    println!("{}", "CPU halted.".magenta());
    println!("-- CPU ENDED -------------------------------------------------------------------------------");
    
}

