struct Registers{
    a:u8,
    b:u8,
    c:u8,
    d:u8,
    e:u8,
    f:u8,
    h:u8,
    l:u8,
} 

impl Registers {
    fn get_bc(&self)-> u16 {
        (self.b as u16) << 8
        | self.c as u16
    }

    fn set_bc(&mut self, value:u16){
        self.b = ((value & 0xFF00)) >> 8 as u8;
        self.c = (value & 0xFF) as u8;
    }
}

struct FlagsRegister{
    zero: bool,
    subtract: bool,
    half_carry:bool,
    carry:bool,
}

struct CPU { 
    registers: Registers,
    pc:u16,
    bus:MemoryBus,
}

struct MemoryBus {
    memory: [u8; 0xFFFF]
}

impl MemoryBus{
    fn read_byte(&self, address: u16) -> u8{
        self.memory[address as usize]
    }
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBSTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8 {
    fn from (byte: u8) -> Self {
        (if flag.zero   {1} else {0}) << ZERO_FLAG_BYTE_POSITION |
        (if flag.subtract  {1} else {0}) << SUBSTRACT_FLAG_BYTE_POSITION |
        (if flag.half_carry   {1} else {0}) << HALF_CARRY_FLAG_BYTE_POSITION |
        (if flag.carry   {1} else {0}) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBSTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry
        }
    }

}

enum Instruction {
    ADD(ArithmeticTaget),
    JP(JumpTest),
}

enum ArithmeticTaget{
    A,B,C,D,E,H,L,
}

enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl CPU {
    fn execute(&mut self, instruction: Instruction){
        match instruction {
            Instruction::ADD(target) =>{
                match target {
                    ArithmeticTaget::C =>{
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        self.pc.wrapping_add(1)
                    }
                    _=>{self.pc}
                }
            }

            Instruction::JP(test)=>{
                let jump_condition = match test{
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => !self.registers.f.zero,
                    JumpTest::Carry => !self.registers.f.carry,
                    JumpTest::Always => true
                };
                self.jump(jump_condition)
            }
            _=> {self.pc}
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    fn step(&mut self){
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }
        
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte,prefixed){
            self.execute(instruction)
        }else {
            let description = format!("0x{}{:x}", if prefixed {"cb"} else {""}, instruction_byte);
            panic!("Unkown instruction found for: Ox{:x}", description);
        };

        self.pc = next_pc;
    }

    fn jump(&self, should_jump: bool) -> u16{
        if should_jump {
            let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
            (most_significant_byte << 8) | least_significant_byte
        }else{
            self.pc.wrapping_add(3)
        }
    }

    fn from_byte(byte: u8) -> Option<Instruction>{
        match byte {
            0x02 => Some(Instruction::INC(IncDecTarget::BC)),
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            _=> None
        }
    }
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        }else{
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed (byte:u8) -> Option <Instruction>{
        match byte{
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            _=> None
        }
    }

    fn from_byte_prefixed (byte:u8) -> Option <Instruction>{
        match byte{
            0x02 => Some(Instruction::RLC(PrefixTarget::BC)),
            _=> None
        }
    }
}
