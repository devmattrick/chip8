pub struct VM {
    // Memory
    memory: [u8; 4096],

    // Stack
    stack: [u8; 64],
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
}

impl VM {
    // Initialize a new virtual machine
    pub fn new() -> VM {
        VM {
            memory: [0; 4096],
            stack: [0; 64],
            stack_pointer: 0,
            program_counter: 0x200,
            registers: [0; 16],
            index_register: 0,
            framebuffer: Framebuffer::new(),
            delay_timer: 0,
            sound_timer: 0,
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
        self.op(opcode);

        // Increment the program counter (since instructions are 2 bytes long, we increment by 2)
        self.program_counter += 2;
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

        match parts {
            // CLS
            (0, 0, 0xE, 0) => (),
            // RET
            (0, 0, 0xE, 0xE) => (),
            // SYS addr
            (0, _, _, _) => (),
            // JP addr
            (1, _, _, _) => (),
            // CALL addr
            (2, _, _, _) => (),
            // SE Vx, byte
            (3, _, _, _) => (),
            // SNE Vx, byte
            (4, _, _, _) => (),
            // SE Vx, Vy
            (5, _, _, _) => (),
            // LD Vx, byte
            (6, _, _, _) => (),
            // ADD Vx, byte
            (7, _, _, _) => (),
            // LD Vx, Vy
            (8, _, _, 0) => (),
            // OR Vx, Vy
            (8, _, _, 1) => (),
            // AND Vx, Vy
            (0, _, _, 2) => (),
            // XOR Vx, Vy
            (8, _, _, 3) => (),
            // ADD Vx, Vy
            (8, _, _, 4) => (),
            // SUB Vx, Vy
            (8, _, _, 5) => (),
            // SHR Vx, Vy
            (8, _, _, 6) => (),
            // SUBN Vx, Vy
            (8, _, _, 7) => (),
            // SHL Vx, Vy
            (8, _, _, 8) => (),
            // SNE Vx, Vy
            (9, _, _, 0) => (),
            // LD I, addr
            (0xA, _, _, _) => (),
            // JP V0, addr
            (0xB, _, _, _) => (),
            // RND Vx, byte
            (0xC, _, _, _) => (),
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => (),
            // SKP Vx
            (0xE, _, 9, 0xE) => (),
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => (),
            // LD Vx, DT
            (0xF, _, 0, 7) => (),
            // LD Vx, K
            (0xF, _, 0, 0xA) => (),
            // LD DT, Vx
            (0xF, _, 1, 5) => (),
            // LD ST, Vx
            (0xF, _, 1, 8) => (),
            // ADD I, Vx
            (0xF, _, 1, 0xE) => (),
            // LD F, Vx
            (0xF, _, 2, 9) => (),
            // LD B, Vx
            (0xF, _, 3, 3) => (),
            // LD [I], Vx
            (0xF, _, 5, 5) => (),
            // LD Vx, [I]
            (0xF, _, 6, 5) => (),

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

    // Set a pixel at (x, y) to be on or off
    fn set(&mut self, x: u8, y: u8, state: bool) {
        // Convert varaibles to proper types for array indexing and bitwise ops
        let y = y as usize;
        let x = x as u64;
        let state = state as u64;

        self.buffer[y] = self.buffer[y] | (state << x);
    }

    // Get a current pixel's state
    fn get(&self, x: u8, y: u8) -> bool {
        // Convert varaibles to proper types for array indexing and bitwise ops
        let y = y as usize;
        let x = x as u64;

        return ((self.buffer[y] >> x) & 0x0001) != 0;
    }
}
