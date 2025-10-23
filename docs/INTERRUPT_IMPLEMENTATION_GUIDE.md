# Game Boy Interrupt System Implementation Guide

## Overview

This guide provides step-by-step instructions for implementing the Game Boy interrupt system in the Rust emulator. The interrupt system allows hardware events (like timer overflows) to temporarily pause execution and jump to handler routines.

**Prerequisites**: Timer system must be implemented and generating interrupts (setting IF register bit 2).

## Implementation Checklist

- [ ] Add `ei_delay` field to CPU struct
- [ ] Update CPU initialization
- [ ] Implement `check_interrupts()` method
- [ ] Implement `handle_interrupt()` method
- [ ] Update `EI` instruction for delayed enable
- [ ] Update `DI` instruction (already correct)
- [ ] Update `RETI` instruction to enable interrupts
- [ ] Update `execute()` to handle EI delay
- [ ] Integrate interrupt checking into `GameBoy::step()`
- [ ] Handle HALT wake-up behavior
- [ ] Write comprehensive tests
- [ ] Validate with test ROMs

## Step 1: Update CPU Structure

### Current State
```rust
pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub halted: bool,
    pub interrupts_enabled: bool,
}
```

### Add EI Delay Field
```rust
pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub halted: bool,
    pub interrupts_enabled: bool, // IME - Interrupt Master Enable
    ei_delay: bool, // EI enables interrupts with 1 cycle delay
}
```

**Rationale**: The `EI` instruction enables interrupts AFTER the next instruction executes. We need a flag to track this delay state.

### Update CPU::new()
```rust
pub fn new() -> Self {
    Self {
        registers: Registers::new(),
        pc: 0x0100,
        sp: 0xFFFE,
        halted: false,
        interrupts_enabled: false,
        ei_delay: false, // Add this line
    }
}
```

## Step 2: Implement Interrupt Checking

Add a method to check if any interrupts are pending and return the highest priority one:

```rust
/// Check if any interrupts are pending and return the vector address
/// Returns Some(vector) if an interrupt should be serviced, None otherwise
fn check_interrupts(&self, memory: &Memory) -> Option<u16> {
    // Only check if IME is enabled
    if !self.interrupts_enabled {
        return None;
    }

    // Read IF (0xFF0F) and IE (0xFFFF)
    let if_flags = memory.read_byte(0xFF0F);
    let ie_flags = memory.read_byte(0xFFFF);

    // Check which interrupts are both requested (IF) and enabled (IE)
    let pending = if_flags & ie_flags & 0x1F; // Mask to 5 bits

    if pending == 0 {
        return None;
    }

    // Check priority order (bit 0 highest, bit 4 lowest)
    if pending & 0x01 != 0 {
        Some(0x0040) // V-Blank
    } else if pending & 0x02 != 0 {
        Some(0x0048) // LCD STAT
    } else if pending & 0x04 != 0 {
        Some(0x0050) // Timer
    } else if pending & 0x08 != 0 {
        Some(0x0058) // Serial
    } else if pending & 0x10 != 0 {
        Some(0x0060) // Joypad
    } else {
        None
    }
}
```

**Key points**:
- Only check when IME is enabled (`interrupts_enabled`)
- Calculate `pending = IF & IE` to find interrupts that are both requested and enabled
- Return vector address based on priority (lower bit = higher priority)
- Mask to 5 bits (0x1F) since only bits 0-4 are used

## Step 3: Implement Interrupt Dispatch

Add a method to handle the actual interrupt dispatch (push PC, jump to vector, disable IME):

```rust
/// Dispatch an interrupt: push PC to stack, jump to vector, disable IME
/// Returns the cycle count (20 cycles for normal dispatch)
fn handle_interrupt(&mut self, vector: u16, memory: &mut Memory) -> u8 {
    // Disable interrupts (IME = 0)
    self.interrupts_enabled = false;

    // Push PC onto stack (high byte first)
    self.sp = self.sp.wrapping_sub(1);
    memory.write_byte(self.sp, (self.pc >> 8) as u8);
    self.sp = self.sp.wrapping_sub(1);
    memory.write_byte(self.sp, (self.pc & 0xFF) as u8);

    // Jump to interrupt vector
    self.pc = vector;

    // Interrupt dispatch takes 20 cycles (5 machine cycles)
    // 2 wait states + 2 for push + 1 for jump
    20
}
```

**Key points**:
- Disable IME immediately (`interrupts_enabled = false`)
- Push current PC to stack (same as CALL instruction)
- Set PC to vector address (0x40, 0x48, 0x50, 0x58, or 0x60)
- Return 20 cycles (interrupt dispatch timing)
- Do NOT clear the IF bit - that's the handler's responsibility

## Step 4: Update EI Instruction

The `EI` instruction must set the delay flag instead of enabling interrupts immediately:

### Current Implementation (INCORRECT)
```rust
fn ei(&mut self) -> u8 {
    self.interrupts_enabled = true; // TOO IMMEDIATE!
    4
}
```

### Correct Implementation
```rust
fn ei(&mut self) -> u8 {
    // EI enables interrupts AFTER the next instruction
    self.ei_delay = true;
    4
}
```

**Rationale**: This allows patterns like `EI` followed by `RET` to work safely - the RET executes before interrupts are enabled.

## Step 5: Update RETI Instruction

The `RETI` instruction must enable interrupts immediately (no delay):

### Current Implementation
```rust
fn reti(&mut self, memory: &Memory) -> u8 {
    let low = memory.read_byte(self.sp);
    self.sp = self.sp.wrapping_add(1);
    let high = memory.read_byte(self.sp);
    self.sp = self.sp.wrapping_add(1);
    self.pc = (u16::from(high) << 8) | u16::from(low);
    // TODO: Enable interrupts when interrupt system is implemented
    16
}
```

### Updated Implementation
```rust
fn reti(&mut self, memory: &Memory) -> u8 {
    let low = memory.read_byte(self.sp);
    self.sp = self.sp.wrapping_add(1);
    let high = memory.read_byte(self.sp);
    self.sp = self.sp.wrapping_add(1);
    self.pc = (u16::from(high) << 8) | u16::from(low);

    // Enable interrupts immediately (no delay like EI)
    self.interrupts_enabled = true;

    16
}
```

## Step 6: Update Main Execute Method

The `execute()` method needs to handle the EI delay. This happens AFTER instruction execution:

### Location
Find the main `execute()` method in `cpu/mod.rs`. It should look something like:
```rust
pub fn execute(&mut self, memory: &mut Memory) -> u8 {
    let opcode = self.fetch_byte(memory);
    instructions::execute_opcode(self, opcode, memory)
}
```

### Updated Implementation
```rust
pub fn execute(&mut self, memory: &mut Memory) -> u8 {
    // Execute the instruction
    let opcode = self.fetch_byte(memory);
    let cycles = instructions::execute_opcode(self, opcode, memory);

    // Handle EI delay - enable interrupts AFTER instruction execution
    if self.ei_delay {
        self.ei_delay = false;
        self.interrupts_enabled = true;
    }

    cycles
}
```

**Key points**:
- After executing ANY instruction, check if `ei_delay` is set
- If set, clear the delay flag and enable interrupts
- This ensures interrupts are enabled after the next instruction, not immediately

## Step 7: Integrate Interrupts into GameBoy::step()

The main game loop needs to check for interrupts and handle HALT behavior.

### Current Implementation (gameboy/mod.rs)
```rust
pub fn step(&mut self) {
    // ... logging code ...

    // Execute instruction
    let cycles = self.cpu.execute(&mut self.memory);
    let timer_interrupt = self.memory.timer.tick(cycles);
    if timer_interrupt {
        let if_register = self.memory.read_byte(0xFF0F);
        self.memory.write_byte(0xFF0F, if_register | 0x04);
        // TODO: Implement interrupt system
    }
}
```

### Updated Implementation
```rust
pub fn step(&mut self) {
    // ... logging code (unchanged) ...

    // Check for interrupts BEFORE execution
    // This handles HALT wake-up
    let if_flags = self.memory.read_byte(0xFF0F);
    let ie_flags = self.memory.read_byte(0xFFFF);
    let pending_interrupts = if_flags & ie_flags & 0x1F;

    // If halted and an interrupt is pending, wake up
    if self.cpu.halted && pending_interrupts != 0 {
        self.cpu.halted = false;
        // HALT wake-up takes 4 cycles, but we'll add those below
    }

    // Execute instruction (if not halted)
    let mut cycles = if !self.cpu.halted {
        self.cpu.execute(&mut self.memory)
    } else {
        4 // HALT continues to consume cycles
    };

    // Tick timer with instruction cycles
    let timer_interrupt = self.memory.timer.tick(cycles);
    if timer_interrupt {
        let if_register = self.memory.read_byte(0xFF0F);
        self.memory.write_byte(0xFF0F, if_register | 0x04);
    }

    // Check if an interrupt should be serviced
    if let Some(vector) = self.cpu.check_interrupts(&self.memory) {
        // Handle the interrupt (push PC, jump to vector, disable IME)
        let interrupt_cycles = self.cpu.handle_interrupt(vector, &mut self.memory);
        cycles = interrupt_cycles; // Replace instruction cycles with interrupt cycles
    }
}
```

**Key points**:
- Check for pending interrupts BEFORE instruction execution (for HALT wake-up)
- If halted and interrupt pending, wake up (regardless of IME)
- Execute instruction normally (or consume 4 cycles if still halted)
- Timer ticks with cycle count (unchanged)
- AFTER instruction execution, check if interrupt should be serviced
- If servicing interrupt, replace cycle count with interrupt dispatch cycles (20)

### Alternative: Simpler Implementation

If you want to start simpler and add HALT behavior later:

```rust
pub fn step(&mut self) {
    // ... logging code (unchanged) ...

    // Execute instruction
    let cycles = self.cpu.execute(&mut self.memory);

    // Tick timer
    let timer_interrupt = self.memory.timer.tick(cycles);
    if timer_interrupt {
        let if_register = self.memory.read_byte(0xFF0F);
        self.memory.write_byte(0xFF0F, if_register | 0x04);
    }

    // Check and handle interrupts
    if let Some(vector) = self.cpu.check_interrupts(&self.memory) {
        self.cpu.handle_interrupt(vector, &mut self.memory);
        // Note: Not adding interrupt cycles to timer here - simplification
    }
}
```

This simpler version:
- Doesn't handle HALT wake-up correctly
- Doesn't add interrupt dispatch cycles to timer
- Good for initial testing

## Step 8: Testing Strategy

### Unit Tests for Interrupt Logic

Add tests to `src/gameboy/mod.rs`:

#### Test 1: Interrupt Triggers When Enabled
```rust
#[test]
fn interrupt_triggers_when_enabled() {
    let mut gb = GameBoy::new();

    // Enable interrupts
    gb.cpu.interrupts_enabled = true;

    // Set timer interrupt flag (IF bit 2)
    gb.memory.write_byte(0xFF0F, 0x04);

    // Enable timer interrupt (IE bit 2)
    gb.memory.write_byte(0xFFFF, 0x04);

    // Put a NOP at current PC
    let current_pc = gb.cpu.pc;
    gb.memory.data[current_pc as usize] = 0x00; // NOP

    // Step once - should service the interrupt
    gb.step();

    // PC should now be at timer interrupt vector (0x0050)
    assert_eq!(gb.cpu.pc, 0x0050, "PC should be at timer interrupt vector");

    // IME should be disabled after interrupt
    assert_eq!(gb.cpu.interrupts_enabled, false, "IME should be disabled after interrupt");
}
```

#### Test 2: Interrupt Doesn't Trigger When IME Disabled
```rust
#[test]
fn interrupt_blocked_when_ime_disabled() {
    let mut gb = GameBoy::new();

    // Keep interrupts disabled (IME = 0)
    gb.cpu.interrupts_enabled = false;

    // Set timer interrupt flag and enable
    gb.memory.write_byte(0xFF0F, 0x04);
    gb.memory.write_byte(0xFFFF, 0x04);

    let current_pc = gb.cpu.pc;
    gb.memory.data[current_pc as usize] = 0x00; // NOP

    // Step once
    gb.step();

    // PC should NOT be at interrupt vector
    assert_ne!(gb.cpu.pc, 0x0050, "Interrupt should not trigger when IME disabled");
}
```

#### Test 3: Interrupt Doesn't Trigger When IE Bit Disabled
```rust
#[test]
fn interrupt_blocked_when_ie_disabled() {
    let mut gb = GameBoy::new();

    // Enable IME
    gb.cpu.interrupts_enabled = true;

    // Set timer interrupt flag but DON'T enable in IE
    gb.memory.write_byte(0xFF0F, 0x04);
    gb.memory.write_byte(0xFFFF, 0x00); // IE = 0, all interrupts disabled

    let current_pc = gb.cpu.pc;
    gb.memory.data[current_pc as usize] = 0x00; // NOP

    gb.step();

    // Should not trigger
    assert_ne!(gb.cpu.pc, 0x0050, "Interrupt should not trigger when IE bit disabled");
}
```

#### Test 4: EI Delay Behavior
```rust
#[test]
fn ei_enables_interrupts_with_delay() {
    let mut gb = GameBoy::new();

    // Set up interrupt condition
    gb.memory.write_byte(0xFF0F, 0x04); // IF bit 2 set
    gb.memory.write_byte(0xFFFF, 0x04); // IE bit 2 set

    // Put EI instruction at PC, followed by NOP, followed by NOP
    gb.memory.data[gb.cpu.pc as usize] = 0xFB;     // EI
    gb.memory.data[gb.cpu.pc as usize + 1] = 0x00; // NOP
    gb.memory.data[gb.cpu.pc as usize + 2] = 0x00; // NOP

    // Execute EI
    gb.step();

    // IME should still be disabled after EI
    assert_eq!(gb.cpu.interrupts_enabled, false, "IME should not be enabled immediately after EI");

    // Execute NOP (the delay instruction)
    gb.step();

    // NOW IME should be enabled
    assert_eq!(gb.cpu.interrupts_enabled, true, "IME should be enabled after delay");

    // Next step should trigger interrupt
    gb.step();
    assert_eq!(gb.cpu.pc, 0x0050, "Interrupt should trigger after EI delay");
}
```

#### Test 5: Interrupt Priority
```rust
#[test]
fn interrupt_priority_order() {
    let mut gb = GameBoy::new();

    gb.cpu.interrupts_enabled = true;

    // Set multiple interrupt flags (timer bit 2 and joypad bit 4)
    gb.memory.write_byte(0xFF0F, 0x14); // Bits 2 and 4
    gb.memory.write_byte(0xFFFF, 0x1F); // All interrupts enabled

    gb.memory.data[gb.cpu.pc as usize] = 0x00; // NOP

    gb.step();

    // Should service timer (bit 2, vector 0x0050) before joypad (bit 4, vector 0x0060)
    assert_eq!(gb.cpu.pc, 0x0050, "Should service higher priority interrupt first");
}
```

#### Test 6: RETI Enables Interrupts Immediately
```rust
#[test]
fn reti_enables_interrupts_immediately() {
    let mut gb = GameBoy::new();

    // Disable interrupts
    gb.cpu.interrupts_enabled = false;

    // Set up stack with return address
    gb.cpu.sp = 0xFFFE;
    gb.memory.write_byte(0xFFFE, 0x34); // Low byte of return address
    gb.memory.write_byte(0xFFFF, 0x12); // High byte of return address

    // Put RETI instruction at PC
    gb.memory.data[gb.cpu.pc as usize] = 0xD9; // RETI

    gb.step();

    // IME should be enabled immediately (no delay like EI)
    assert_eq!(gb.cpu.interrupts_enabled, true, "RETI should enable interrupts immediately");
    assert_eq!(gb.cpu.pc, 0x1234, "RETI should return to correct address");
}
```

#### Test 7: PC Pushed to Stack Correctly
```rust
#[test]
fn interrupt_pushes_pc_to_stack() {
    let mut gb = GameBoy::new();

    gb.cpu.interrupts_enabled = true;
    gb.cpu.pc = 0x1234;
    gb.cpu.sp = 0xFFFE;

    gb.memory.write_byte(0xFF0F, 0x04); // Timer interrupt
    gb.memory.write_byte(0xFFFF, 0x04);

    gb.memory.data[0x1234] = 0x00; // NOP at PC

    gb.step();

    // Check that PC was pushed to stack
    assert_eq!(gb.cpu.sp, 0xFFFC, "SP should decrease by 2");
    assert_eq!(gb.memory.read_byte(0xFFFC), 0x34, "Low byte of PC should be on stack");
    assert_eq!(gb.memory.read_byte(0xFFFD), 0x12, "High byte of PC should be on stack");
}
```

### Integration Test with Timer

Add a test that verifies the full timer interrupt flow:

```rust
#[test]
fn timer_overflow_triggers_interrupt_dispatch() {
    let mut gb = GameBoy::new();

    // Enable interrupts
    gb.cpu.interrupts_enabled = true;

    // Enable timer interrupt in IE
    gb.memory.write_byte(0xFFFF, 0x04);

    // Setup timer to overflow quickly
    gb.memory.write_byte(0xFF07, 0x05); // Enable timer, frequency 16
    gb.memory.write_byte(0xFF05, 0xFF); // TIMA = 0xFF
    gb.memory.write_byte(0xFF06, 0x00); // TMA = 0x00

    // Clear IF
    gb.memory.write_byte(0xFF0F, 0x00);

    // Put NOPs at PC
    for i in 0..10 {
        gb.memory.data[gb.cpu.pc as usize + i] = 0x00;
    }

    let initial_pc = gb.cpu.pc;

    // Execute 4 NOPs = 16 cycles, causing timer overflow
    for _ in 0..4 {
        gb.step();
    }

    // PC should have jumped to interrupt vector
    assert_eq!(gb.cpu.pc, 0x0050, "Timer overflow should trigger interrupt");
    assert_eq!(gb.cpu.interrupts_enabled, false, "IME should be disabled after interrupt");

    // Stack should contain return address
    assert_eq!(gb.cpu.sp, 0xFFFC, "PC should be pushed to stack");
}
```

## Step 9: Validation

### Compile and Test
```bash
# Ensure code compiles
cargo build

# Run all tests
cargo test

# Run specific interrupt tests
cargo test interrupt

# Check for warnings
cargo clippy
```

### Expected Results
- All existing tests should still pass (187 tests)
- New interrupt tests should pass
- No clippy warnings

### Test ROM Validation
Once implemented, test with actual Game Boy test ROMs:
- `cpu_instrs.gb` - Tests all instructions including interrupt handling
- `interrupt_time.gb` - Tests interrupt timing accuracy

## Common Issues and Debugging

### Issue 1: Infinite Interrupt Loop
**Symptom**: PC keeps jumping to interrupt vector repeatedly
**Cause**: IF bit not being cleared in the interrupt handler
**Fix**: Software must clear the IF bit. In tests, manually clear it after servicing.

### Issue 2: EI Followed by RET Doesn't Work
**Symptom**: Interrupt triggers during RET instruction
**Cause**: EI enabling interrupts immediately instead of with delay
**Fix**: Ensure `ei_delay` flag is used and interrupts only enable AFTER next instruction

### Issue 3: Interrupts Never Trigger
**Symptom**: IF is set but interrupt never services
**Checklist**:
- Is IME enabled? (`cpu.interrupts_enabled = true`)
- Is IE bit set? (Check 0xFFFF register)
- Is IF bit set? (Check 0xFF0F register)
- Is `check_interrupts()` being called in the main loop?

### Issue 4: Wrong Interrupt Vector
**Symptom**: Jumps to 0x0040 instead of 0x0050
**Cause**: Priority logic incorrect - checking bits in wrong order
**Fix**: Check bit 0 (V-Blank) first, then bit 1, then bit 2, etc.

### Issue 5: HALT Never Wakes Up
**Symptom**: CPU stuck in HALT even with interrupts pending
**Cause**: Not checking for pending interrupts to wake from HALT
**Fix**: Check `IF & IE != 0` before instruction execution and set `halted = false`

## Architecture Summary

After implementation, the interrupt flow will be:

```
1. Hardware event occurs (e.g., timer overflow)
   ↓
2. Hardware sets bit in IF register (0xFF0F)
   ↓
3. GameBoy::step() executes CPU instruction
   ↓
4. GameBoy::step() calls cpu.check_interrupts()
   ↓
5. check_interrupts() checks: IME && (IF & IE)
   ↓
6. If interrupt pending: cpu.handle_interrupt(vector)
   ↓
7. handle_interrupt() pushes PC, jumps to vector, disables IME
   ↓
8. Interrupt handler executes at vector address
   ↓
9. Handler clears IF bit (prevents re-trigger)
   ↓
10. Handler executes RETI
   ↓
11. RETI pops PC, enables IME
   ↓
12. Execution resumes where interrupted
```

## Next Steps After Interrupts

Once the interrupt system is complete:
1. Implement V-Blank interrupt (when PPU finishes frame)
2. Implement LCD STAT interrupt (various LCD conditions)
3. Implement joypad interrupt (button press)
4. Start PPU (Picture Processing Unit) implementation

## References
- `INTERRUPT_REFERENCE.md` - How interrupts work
- `The Cycle-Accurate Game Boy Docs.pdf` - Pages 14-17
- Pan Docs Interrupts: https://gbdev.io/pandocs/Interrupts.html
