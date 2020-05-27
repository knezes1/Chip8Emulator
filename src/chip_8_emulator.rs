use std::fs::File;
use std::io::prelude::*;
use rand::Rng;

type BYTE = u8;     // 8bit -> 1 byte
type WORD = u16;    // 16 bit -> 1 word

pub struct Chip8Hardware {
    pub keyboard: [bool; 16],
    address_i: WORD,
    program_counter: WORD,
    registers: [BYTE; 16],
    memory: [BYTE; 0xFFF],
    stack: [WORD; 16],
    stack_pointer: usize,
    fontset: [BYTE; 80],
    // 32 is length, 64 is height
    screen_data: [[BYTE; 64]; 32],
    delay_timer: BYTE,
    sound_timer: BYTE,
    timer_counter: BYTE,
    draw_enabled: bool,
}

impl Chip8Hardware{
    pub fn cpu_reset(&mut self){
        self.memory = [0; 0xFFF];
        self.screen_data = [[0; 64]; 32];
        self.stack = [0; 16];
        self.stack_pointer = 0;
        self.keyboard = [false; 16];        // true if pressed, false if not pressed
        self.address_i = 0;
        self.registers = [0; 16];       // set all registers to 0
        self.program_counter = 0x200;
        self.timer_counter = 30;
        self.draw_enabled = false;

        self.fontset =
        [ 
          0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
          0x20, 0x60, 0x20, 0x20, 0x70, // 1
          0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
          0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
          0x90, 0x90, 0xF0, 0x10, 0x10, // 4
          0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
          0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
          0xF0, 0x10, 0x20, 0x40, 0x40, // 7
          0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
          0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
          0xF0, 0x90, 0xF0, 0x90, 0x90, // A
          0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
          0xF0, 0x80, 0x80, 0x80, 0xF0, // C
          0xE0, 0x90, 0x90, 0x90, 0xE0, // D
          0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
          0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for i in 0..80{
            self.memory[i] = self.fontset[i];
        }
    }

    pub fn new() -> Chip8Hardware{
        Chip8Hardware{
            memory: [0; 0xFFF],
            screen_data: [[0; 64]; 32],
            stack: [0; 16],
            stack_pointer: 0,
            keyboard: [false; 16],        // 1 if pressed, 0 if not pressed
            address_i: 0,
            registers: [0; 16],       // set all registers to 0
            program_counter: 0x200,
            draw_enabled: false,
            fontset:
            [ 
              0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
              0x20, 0x60, 0x20, 0x20, 0x70, // 1
              0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
              0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
              0x90, 0x90, 0xF0, 0x10, 0x10, // 4
              0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
              0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
              0xF0, 0x10, 0x20, 0x40, 0x40, // 7
              0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
              0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
              0xF0, 0x90, 0xF0, 0x90, 0x90, // A
              0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
              0xF0, 0x80, 0x80, 0x80, 0xF0, // C
              0xE0, 0x90, 0x90, 0x90, 0xE0, // D
              0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
              0xF0, 0x80, 0xF0, 0x80, 0x80  // F
            ],
            delay_timer: 0,
            sound_timer: 0,
            timer_counter: 30,
        }
    }

    pub fn load_game(&mut self, game_file: String){
        let mut f = File::open(game_file).unwrap();
        let mut file_buffer = [0; 0xFFF - 0x200];
        let buffer_size = f.read(&mut file_buffer[..]).expect("Error reading file");

        for i in 0..buffer_size {
            self.memory[i + 0x200] = file_buffer[i];
        }
    }

    pub fn fetch_opcode(&mut self, opcode_value: &mut WORD){
        *opcode_value = self.memory[self.program_counter as usize] as u16;
        *opcode_value <<= 8;
        *opcode_value |= (self.memory[self.program_counter as usize + 1]) as u16;
        self.program_counter += 2;
    }

    pub fn decode_and_execute_opcode(&mut self, opcode: WORD){
        match opcode & 0xF000{                              // switch on first segment of opcode X _ _ _
            0x1000 => Chip8Hardware::opcode_1NNN(self, opcode),   // jump opcode
            0x0000 => {
                match opcode & 0x000F{                      // see which opcode that starts with 00 we will need
                    0x0000 => Chip8Hardware::opcode_00E0(self),
                    0x000E => Chip8Hardware::opcode_00EE(self),
                    _ => println!("no opcode that matches!1"),
                }
            }
            0x2000 => Chip8Hardware::opcode_2NNN(self, opcode),
            0x3000 => Chip8Hardware::opcode_3XNN(self, opcode),
            0x4000 => Chip8Hardware::opcode_4XNN(self, opcode),
            0x5000 => Chip8Hardware::opcode_5XY0(self, opcode),                
            0x6000 => Chip8Hardware::opcode_6XNN(self, opcode),            
            0x7000 => Chip8Hardware::opcode_7XNN(self, opcode),   
            0x8000 => {
                match opcode & 0x000F{
                    0x0000 => Chip8Hardware::opcode_8XY0(self, opcode),
                    0x0001 => Chip8Hardware::opcode_8XY1(self, opcode),
                    0x0002 => Chip8Hardware::opcode_8XY2(self, opcode),
                    0x0003 => Chip8Hardware::opcode_8XY3(self, opcode),
                    0x0004 => Chip8Hardware::opcode_8XY4(self, opcode),
                    0x0005 => Chip8Hardware::opcode_8XY5(self, opcode),
                    0x0006 => Chip8Hardware::opcode_8XY6(self, opcode),
                    0x0007 => Chip8Hardware::opcode_8XY7(self, opcode),
                    0x000E => Chip8Hardware::opcode_8XYE(self, opcode),
                    _ => panic!("{} not covered", opcode),
                }
            },
            0x9000 => Chip8Hardware::opcode_9XY0(self, opcode),
            0xA000 => Chip8Hardware::opcode_ANNN(self, opcode),
            0xB000 => Chip8Hardware::opcode_BNNN(self, opcode),
            0xC000 => Chip8Hardware::opcode_CXNN(self, opcode),
            0xD000 => Chip8Hardware::opcode_DXYN(self, opcode),
            0xE000 => {
                match opcode & 0x000F{
                    0x000E => Chip8Hardware::opcode_EX9E(self, opcode),
                    0x0001 => Chip8Hardware::opcode_EXA1(self, opcode),
                    _ => println!("Not covered"),
                }
            },
            0xF000 => {
                match opcode & 0x00FF{
                    0x0007 => Chip8Hardware::opcode_FX07(self, opcode),
                    0x000A => Chip8Hardware::opcode_FX0A(self, opcode),
                    0x0015 => Chip8Hardware::opcode_FX15(self, opcode),
                    0x0018 => Chip8Hardware::opcode_FX18(self, opcode),
                    0x001E => Chip8Hardware::opcode_FX1E(self, opcode),
                    0x0029 => Chip8Hardware::opcode_FX29(self, opcode),
                    0x0033 => Chip8Hardware::opcode_FX33(self, opcode),
                    0x0055 => Chip8Hardware::opcode_FX55(self, opcode),
                    0x0065 => Chip8Hardware::opcode_FX65(self, opcode),
                    _ => println!("not covered"),
                }
            }
            _ => println!("{} no opcode that matches!2", opcode),
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_1NNN(&mut self, opcode: WORD){
        self.program_counter = opcode & 0x0FFF;
    }

    #[allow(non_snake_case)]
    pub fn opcode_00E0(& mut self){
        self.draw_enabled = true;
        for i in 0..32{
            for j in 0..64{
                self.screen_data[i][j] = 0;
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_00EE(&mut self){
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    #[allow(non_snake_case)]
    pub fn opcode_2NNN(&mut self, opcode: WORD){
        self.stack[self.stack_pointer as usize] = (self.program_counter + 2) as WORD;
        self.stack_pointer += 1;
        self.program_counter = opcode & 0x0FFF;
    }

    #[allow(non_snake_case)]
    pub fn opcode_3XNN(&mut self, opcode: WORD){
    //  skip next instruction if registers[X] == NN
        let mut index: WORD = opcode & 0x0F00;
        index = index >> 8;
        let reg_value: WORD = self.registers[index as usize] as WORD;
        let opcode_value: WORD = opcode & 0x00FF;
        if reg_value == opcode_value{
            Chip8Hardware::skip_instruction(self);
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_4XNN(&mut self, opcode: WORD){
        //skip next instruction if registers[x] != NN
        let mut index: WORD = opcode & 0x0F00;
        index = index >> 8;
        let reg_value: WORD = self.registers[index as usize] as WORD;
        let opcode_value: WORD = opcode & 0x00FF;
        if reg_value != opcode_value{
            Chip8Hardware::skip_instruction(self);
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_5XY0(&mut self, opcode: WORD){
        //skip next instruction if registers[x] == registers[y]
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let mut index_y: WORD = Chip8Hardware::get_second_arg(opcode);
        index_y = index_y >> 4;

        let reg_value_x: WORD = self.registers[index_x as usize] as WORD;
        let reg_value_y: WORD = self.registers[index_y as usize] as WORD;

        if reg_value_x == reg_value_y{
            Chip8Hardware::skip_instruction(self);
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_6XNN(&mut self, opcode: WORD){
        // set registers[x] = NN
        let mut index_x: WORD = opcode & 0x0F00;
        index_x = index_x >> 8;

        let opcode_value: WORD = opcode & 0x00FF;

        self.registers[index_x as usize] = opcode_value as BYTE;
    }

    #[allow(non_snake_case)]
    pub fn opcode_7XNN(&mut self, opcode: WORD){
        // set registers[x] += NN
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let value_x: WORD = Chip8Hardware::get_register_value(self, index_x);
        let NN: WORD = Chip8Hardware::get_nn(opcode);

        Chip8Hardware::set_register_value(self, index_x, value_x + NN);
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY0(&mut self, opcode: WORD){
        // set value of registers[x] = registers[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_y = Chip8Hardware::get_register_value(self, index_y);

        Chip8Hardware::set_register_value(self, index_x, value_y);
    }
    
    #[allow(non_snake_case)]
    pub fn opcode_8XY1(&mut self, opcode: WORD){
        // set value of register[x] = register[x] | register[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        let or_x_y = value_x | value_y;

        Chip8Hardware::set_register_value(self, index_x, or_x_y);
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY2(&mut self, opcode: WORD){
        // set value of register[x] = register[x] & register[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        let and_x_y = value_x & value_y;

        Chip8Hardware::set_register_value(self, index_x, and_x_y);
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY3(&mut self, opcode: WORD){
        // set value of register[x] = register[x] ^ register[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        let xor_x_y = value_x ^ value_y;

        Chip8Hardware::set_register_value(self, index_x, xor_x_y);
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY4(&mut self, opcode: WORD){
        // set value of register[x] = register[x] + register[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        let sum_x_y = value_x + value_y;

        // psuedo ternary for if x + y overflows
        self.registers[0xF] = if sum_x_y > 255 {1} else {0};

        Chip8Hardware::set_register_value(self, index_x, sum_x_y);  
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY5(&mut self, opcode: WORD){
        // set value of register[x] = register[x] - register[y]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);
        //need to do wrapping sub here
        let diff_x_y = value_x.wrapping_sub(value_y);

        // psuedo ternary for if x - y < 0
        self.registers[0xF] = if value_y > value_x {0} else {1};

        Chip8Hardware::set_register_value(self, index_x, diff_x_y);         
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY6(&mut self, opcode: WORD){
        // set registers[0xFF] = leastSignificantBit(registers[X])
        // then registers[X] = registers[X] >> 1

        // it's the index we need to reference, but also the value
        let index_x = Chip8Hardware::get_first_arg(opcode);
        // index_x & 0x01 gives least significant bit
        Chip8Hardware::set_register_value(self, 0xF, index_x & 0x01);
        let shifted_index_x = index_x >> 1;
        Chip8Hardware::set_register_value(self, index_x, shifted_index_x);
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XY7(&mut self, opcode: WORD){
        // set value of register[x] = register[y] - register[x]
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        let diff_x_y = value_y - value_x;

        // psuedo ternary for if y - x < 0
        self.registers[0xF] = if value_y < value_x {0} else {1};

        Chip8Hardware::set_register_value(self, index_x, diff_x_y);      
    }

    #[allow(non_snake_case)]
    pub fn opcode_8XYE(&mut self, opcode: WORD){
        // set registers[0xFF] = mostSignificantBit(registers[X])
        // then registers[X] = registers[X] << 1

        // it's the index we need to reference, but also the value
        let index_x = Chip8Hardware::get_first_arg(opcode);

        // >> 7 gets most significant bit
        Chip8Hardware::set_register_value(self, 0xF, index_x >> 7);
        let shifted_index_x = index_x << 1;
        Chip8Hardware::set_register_value(self, index_x, shifted_index_x);        
    }

    #[allow(non_snake_case)]
    pub fn opcode_9XY0(&mut self, opcode: WORD){
        let index_x = Chip8Hardware::get_first_arg(opcode);
        let index_y = Chip8Hardware::get_second_arg(opcode);

        let value_x = Chip8Hardware::get_register_value(self, index_x);
        let value_y = Chip8Hardware::get_register_value(self, index_y);

        if value_x != value_y{
            Chip8Hardware::skip_instruction(self);
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_ANNN(&mut self, opcode: WORD){
        let nnn: WORD = Chip8Hardware::get_nnn(opcode);
        self.address_i = nnn;
    }

    #[allow(non_snake_case)]
    pub fn opcode_BNNN(&mut self, opcode: WORD){
        let nnn: WORD = Chip8Hardware::get_nnn(opcode);
        let register_value_0: WORD = Chip8Hardware::get_register_value(self, 0);

        self.program_counter = nnn + register_value_0;
    }

    #[allow(non_snake_case)]
    pub fn opcode_CXNN(& mut self, opcode: WORD){
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);

        let nn: WORD = Chip8Hardware::get_nn(opcode);

        let random_number: WORD = rand::thread_rng().gen_range(0,256);

        Chip8Hardware::set_register_value(self, index_x, nn & random_number);
    }

    #[allow(non_snake_case)]
    pub fn opcode_DXYN(&mut self, opcode: WORD){
        self.draw_enabled = true;
        // drawing to screen, if any bits are flipped, set registers[15] = 1

        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let value_x: WORD = Chip8Hardware::get_register_value(self, index_x);
        let index_y: WORD = Chip8Hardware::get_second_arg(opcode);
        let value_y: WORD = Chip8Hardware::get_register_value(self, index_y);
        let height: WORD = Chip8Hardware::get_n(opcode);

        // set registers[15] = 0
        Chip8Hardware::set_register_value(self, 0xF, 0);

        // how many lines of height we are rendering
        for y in 0..height{

            // get the pixel which is a byte which is 8 bit _ _ _ _ _ _ _ _ -> these are 0s and 1s
            // this pixel is stored in the  game memory array starting at address i
            // and goes on until the height of the thing you're rendering is reached
            let pixel = self.memory[(self.address_i + y) as usize];

            // for each of the 8 bytes in the pixel we found above
            for x in 0..8{

                // we're going to do a bitwise and on that pixel with the value 0x80
                // this is to say _ _ _ _ _ _ _ _ & 1 0 0 0 0 0 0 0
                // but this 0x80 will be right shifted on each iteration
                // so itll look like this
                // 10000000
                // 01000000
                // 00100000
                //  ...
                // 00000001
                // which is to say we're going to do an and for each bit in this byte of pixel data 
                // if that &'ed pixel is not equal to 0, there is something there
                if (pixel & (0x80 >> x)) != 0{

                    // I found this online, this is for index out of bounds I guess
                    // if index_x + x >= 32{
                    //     continue;
                    // }

                    // if the screen data for that pixel == 1, we set register[15] to 1
                    if self.screen_data[((value_y + y) % 32) as usize][((value_x + x) % 64) as usize] == 1{
                        // set registers[15] = 1
                        Chip8Hardware::set_register_value(self, 0xF, 1);
                    }

                    // flip the bit by xor'ing the screen data we have for it here
                    self.screen_data[((value_y + y) % 32) as usize][((value_x + x) % 64) as usize] ^= 1;
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_EX9E(&mut self, opcode: WORD){
        let index: WORD = Chip8Hardware::get_first_arg(opcode);
        let value_x = self.get_register_value(index);
        if self.keyboard[value_x as usize] {
            Chip8Hardware::skip_instruction(self);
        }

    }

    #[allow(non_snake_case)]
    pub fn opcode_EXA1(&mut self, opcode: WORD){
        let index: WORD = Chip8Hardware::get_first_arg(opcode);
        let value_x = self.get_register_value(index);
        if !self.keyboard[value_x as usize] {
            Chip8Hardware::skip_instruction(self);
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX07(&mut self, opcode: WORD){
        // set register[X] = delay_timer
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        Chip8Hardware::set_register_value(self, index_x, self.delay_timer as WORD);

    }

    #[allow(non_snake_case)]
    pub fn opcode_FX0A(&mut self, opcode: WORD){
        // wait for keypress, 
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let mut key_pressed: bool = false;

        for i in 0..16{
            if self.keyboard[i] {
                Chip8Hardware::set_register_value(self, index_x, i as WORD);
                key_pressed = true;
            }
        }

        if !key_pressed {
            self.program_counter -= 2;
            return;
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX15(&mut self, opcode: WORD){
        // set delay timer to register[X]
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let x: WORD = Chip8Hardware::get_register_value(self, index_x);
        self.delay_timer = x as BYTE;
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX18(&mut self, opcode: WORD){
        // set sound timer to register[X]
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);
        let x: WORD = Chip8Hardware::get_register_value(self, index_x);
        self.sound_timer = x as BYTE;
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX1E(&mut self, opcode: WORD){
        // Add register[x] to address_i, set register[F] to 1 if overflow, 0 otherwise
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);

        // set V[15] = 0 or 1
        Chip8Hardware::set_register_value(self, 0xF, if self.address_i + index_x > 0xFFF { 1 } else { 0 });
        self.address_i += index_x;
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX29(&mut self, opcode: WORD){
        // set address_i to location of the sprite for character in register[X]
        let index: WORD = Chip8Hardware::get_first_arg(opcode);
        self.address_i = self.memory[index as usize] as WORD;
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX33(&mut self, opcode: WORD){
        let index: WORD = Chip8Hardware::get_first_arg(opcode);

        let mut value_x: WORD = Chip8Hardware::get_register_value(self, index);

        let ones: WORD = value_x % 10;
        value_x = value_x / 10;
        let tens: WORD = value_x % 10;
        value_x = value_x / 10;
        let hundreds: WORD = value_x % 10;

        self.memory[self.address_i as usize] = hundreds as BYTE;
        self.memory[(self.address_i + 1) as usize] = tens as BYTE;
        self.memory[(self.address_i + 2) as usize] = ones as BYTE;
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX55(&mut self, opcode: WORD){
        //dump value of registers into memory starting at address i
        
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);        

        for i in 0..index_x + 1 {
            let register_value_i: WORD = Chip8Hardware::get_register_value(self, i);
            self.memory[(self.address_i + i) as usize] = register_value_i as BYTE;
        }
    }

    #[allow(non_snake_case)]
    pub fn opcode_FX65(&mut self, opcode: WORD){
        let index_x: WORD = Chip8Hardware::get_first_arg(opcode);

        for i in 0..index_x + 1{
            let memory_value_i: WORD = self.memory[(self.address_i + i) as usize] as WORD;
            Chip8Hardware::set_register_value(self, i, memory_value_i);        
        }
    }

    #[allow(non_snake_case)]
    pub fn get_first_arg(opcode: WORD) -> WORD{
        let mut argument: WORD = opcode & 0x0F00;
        argument = argument >> 8;
        return argument;
    }

    pub fn get_second_arg(opcode: WORD) -> WORD {
        let mut argument: WORD = opcode & 0x00F0;
        argument = argument >> 4;
        return argument;
    }

    pub fn get_n(opcode: WORD) -> WORD {
        let argument: WORD = opcode & 0x000F;
        return argument;
    }

    pub fn get_nn(opcode: WORD) -> WORD {
        let argument: WORD = opcode & 0x00FF;
        return argument;
    }

    pub fn get_nnn(opcode: WORD) -> WORD {
        let argument: WORD = opcode & 0x0FFF;
        return argument;
    }

    pub fn set_register_value(&mut self, index: WORD, opcode_value: WORD){
        self.registers[index as usize] = opcode_value as BYTE;
    }

    pub fn get_register_value(&mut self, index: WORD) -> WORD {
        return self.registers[index as usize] as WORD;
    }

    pub fn skip_instruction(&mut self){
        self.program_counter += 2;
    }

    pub fn get_pixel_value_x_y(&self, x: u16, y: u16) -> bool{
        return if self.screen_data[x as usize][y as usize] == 1 {true} else {false};
    }

    pub fn decrement_timer_counter(&mut self){
        self.timer_counter -= 1;
        if self.timer_counter == 0{
            self.timer_counter = 30;

            self.sound_timer += 1;
            self.delay_timer += 1;

            if self.sound_timer > 10{
                self.sound_timer = 0;
                // do a sound here?
            }
            if self.delay_timer > 10{
                self.delay_timer = 0;
                // do something else here...?
            }

        }
    }

    pub fn get_draw_enabled(&self) -> bool{
        return self.draw_enabled;
    }

    pub fn disable_draw_enabled(&mut self){
        self.draw_enabled = false;
    }
}