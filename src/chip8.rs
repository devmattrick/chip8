use rand::prelude::*;

pub struct VM {
    // Memory
    memory: [u8; 4096],

    // Stack
    stack: [u16; 16],
    stack_pointer: u8,

    // Program counter
    program_counter: u16,

    // Registers
    registers: [u8; 16],
    index_register: u16,

    // Graphics
    framebuffer: Framebuffer,

    // Timers
    delay_timer: u8,
    sound_timer: u8,

    // Input
    keyboard: [bool; 16],

    // Random
    rng: ThreadRng,
}

impl VM {
    // Initialize a new virtual machine
    pub fn new() -> VM {
        VM {
            memory: [0; 4096],
            stack: [0; 16],
            stack_pointer: 0,
            program_counter: 0x200,
            registers: [0; 16],
            index_register: 0,
            framebuffer: Framebuffer::new(),
            delay_timer: 0,
            sound_timer: 0,
            keyboard: [false; 16],
            rng: thread_rng(),
        }
    }

    // Load a game ROM into memory
    pub fn load(&mut self, rom: [u8; 3584]) {
        for (i, value) in self.memory[512..4095].iter_mut().enumerate() {
            *value = rom[i];
        }
    }

    // Simulate a CPU cycle
    pub fn cycle(&mut self) {
        // Read the next instruction and execute it
        let opcode = self.read_instruction();

        // Increment the program counter (since instructions are 2 bytes long, we increment by 2)
        self.program_counter += 2;

        self.op(opcode);
    }

    pub fn key(&mut self, key: u8, state: bool) {
        if key >= 0xF {
            panic!("Illegal key: {}", key);
        }

        let key = key as usize;

        self.keyboard[key] = state;
    }

    // Read an instruction
    fn read_instruction(&self) -> u16 {
        let index = self.program_counter as usize;

        return ((self.memory[index] as u16) << 8) | (self.memory[index + 1] as u16);
    }

    // Process an opcode
    fn op(&mut self, opcode: u16) {
        // Break the opcode into a tuple of individual nibbles
        let parts = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );

        // Pre compute arguments for ease of use
        let nnn = opcode & 0x0FFF;
        let vx = ((opcode & 0x0F00) >> 8) as usize;
        let vy = ((opcode & 0x00F0) >> 4) as usize;
        let byte = (opcode & 0x00FF) as u8;

        match parts {
            // CLS
            (0, 0, 0xE, 0) => self.framebuffer.clear(),
            // RET
            (0, 0, 0xE, 0xE) => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
            }
            // AND Vx, Vy
            (0, _, _, 2) => self.registers[vx] &= self.registers[vy],
            // SYS addr
            (0, _, _, _) => (),
            // JP addr
            (1, _, _, _) => self.program_counter = nnn,
            // CALL addr
            (2, _, _, _) => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = nnn;
            }
            // SE Vx, byte
            (3, _, _, _) => {
                if self.registers[vx] == byte {
                    self.program_counter += 2;
                }
            }
            // SNE Vx, byte
            (4, _, _, _) => {
                if self.registers[vx] != byte {
                    self.program_counter += 2;
                }
            }
            // SE Vx, Vy
            (5, _, _, _) => {
                if self.registers[vx] == self.registers[vy] {
                    self.program_counter += 2;
                }
            }
            // LD Vx, byte
            (6, _, _, _) => self.registers[vx] = byte,
            // ADD Vx, byte
            (7, _, _, _) => self.registers[vx] += byte,
            // LD Vx, Vy
            (8, _, _, 0) => self.registers[vx] = self.registers[vy],
            // OR Vx, Vy
            (8, _, _, 1) => self.registers[vx] |= self.registers[vy],
            // XOR Vx, Vy
            (8, _, _, 3) => self.registers[vx] ^= self.registers[vy],
            // ADD Vx, Vy
            (8, _, _, 4) => {
                let sum = (self.registers[vx] as u16) + (self.registers[vy] as u16);

                self.registers[0xF] = (sum > 255) as u8;
                self.registers[vx] = sum as u8 & 0xFF;
            }
            // SUB Vx, Vy
            (8, _, _, 5) => {
                self.registers[0xF] = (self.registers[vx] > self.registers[vy]) as u8;
                self.registers[vx] -= self.registers[vy];
            }
            // SHR Vx
            (8, _, _, 6) => {
                self.registers[0xF] = self.registers[vx] & 0x1;
                self.registers[vx] >>= 1;
            }
            // SUBN Vx, Vy
            (8, _, _, 7) => {
                self.registers[0xF] = (self.registers[vy] > self.registers[vx]) as u8;
                self.registers[vx] = self.registers[vy] - self.registers[vx];
            }
            // SHL Vx, Vy
            (8, _, _, 8) => {
                self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;
                self.registers[vx] <<= 1;
            }
            // SNE Vx, Vy
            (9, _, _, 0) => {
                if self.registers[vx] != self.registers[vy] {
                    self.program_counter += 2;
                }
            }
            // LD I, addr
            (0xA, _, _, _) => self.index_register = nnn,
            // JP V0, addr
            (0xB, _, _, _) => self.program_counter = (self.registers[0] as u16) + nnn,
            // RND Vx, byte
            (0xC, _, _, _) => {
                self.registers[vx] = (self.rng.gen_range(0, 255) as u8) & byte;
            }
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let height = (opcode & 0x000F) as u8;

                let x_start = vx as u8 % 64;
                let y_start = vy as u8 % 32;

                let mut collision = false;

                for x in x_start..(x_start + 8) {
                    for y in y_start..(y_start + height) {
                        let x = x as u16;
                        let y = y as u16;

                        let pixel = (self.memory[(self.index_register + y) as usize] & (0x80 >> x)) != 0;

                        collision |= self.framebuffer.set(x as u8, y as u8, pixel);
                    }
                }

                self.registers[0xF] = collision as u8;
            },
            // SKP Vx
            (0xE, _, 9, 0xE) => {
                let key = self.registers[vx] as usize;

                if self.keyboard[key] {
                    self.program_counter += 2;
                }
            },
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                let key = self.registers[vx] as usize;

                if !self.keyboard[key] {
                    self.program_counter += 2;
                }
            },
            // LD Vx, DT
            (0xF, _, 0, 7) => self.registers[vx] = self.delay_timer,
            // LD Vx, K
            (0xF, _, 0, 0xA) => {
                for (i, key) in self.keyboard.iter().enumerate() {
                    if *key {
                        self.registers[vx] = i as u8;
                        return;
                    }
                }

                self.program_counter -= 2;
            },
            // LD DT, Vx
            (0xF, _, 1, 5) => self.delay_timer = self.registers[vx],
            // LD ST, Vx
            (0xF, _, 1, 8) => self.sound_timer = self.registers[vx],
            // ADD I, Vx
            (0xF, _, 1, 0xE) => self.index_register += self.registers[vx] as u16,
            // LD F, Vx
            (0xF, _, 2, 9) => {
                let digit = self.registers[vx];
                self.index_register = (0x50 + (5 * digit)) as u16;
            },
            // LD B, Vx
            (0xF, _, 3, 3) => {
                let mut val = self.registers[vx];
                let index = self.index_register as usize;

                self.memory[index + 2] = val % 10;
                val /= 10;

                self.memory[index + 1] = val % 10;
                val /= 10;

                self.memory[index] = val % 10;
            }
            // LD [I], Vx
            (0xF, _, 5, 5) => {
                for i in 0..vx {
                    let i = i as usize;

                    self.memory[self.index_register as usize + i] = self.registers[i];
                }
            }
            // LD Vx, [I]
            (0xF, _, 6, 5) => {
                for i in 0..vx {
                    let i = i as usize;

                    self.registers[i] = self.memory[self.index_register as usize + i];
                }
            }

            (_, _, _, _) => println!("Unknown opcode: {:#06x}", opcode),
        }
    }
}

// 64x32 bit framebuffer. Essentially a bitfield; abstracted to make accessing individual pixels easier
struct Framebuffer {
    buffer: [u64; 32],
}

impl Framebuffer {
    // Create a new, empty framebuffer
    fn new() -> Framebuffer {
        Framebuffer { buffer: [0; 32] }
    }

    // Clear the framebuffer (set all pixels to off)
    fn clear(&mut self) {
        self.buffer = [0; 32];
    }

    // Set a pixel at (x, y) to be on or off. Returns true if there was a collision.
    fn set(&mut self, x: u8, y: u8, state: bool) -> bool {
        let mut collision = false;
        if self.get(x, y) {
            collision = true;
        }

        // Convert varaibles to proper types for array indexing and bitwise ops
        let y = y as usize;
        let x = x as u64;
        let state = state as u64;

        self.buffer[y] = self.buffer[y] | (state << x);

        return collision;
    }

    // Get a current pixel's state
    fn get(&self, x: u8, y: u8) -> bool {
        // Convert varaibles to proper types for array indexing and bitwise ops
        let y = y as usize;
        let x = x as u64;

        return ((self.buffer[y] >> x) & 0x0001) != 0;
    }
}
