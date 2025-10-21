# Timer Implementation Guide

This guide will walk you through implementing the Game Boy timer system step-by-step.

## Overview

You'll be creating a new `timer` module that tracks:
- DIV register (auto-incrementing divider)
- TIMA register (programmable timer counter)
- TMA register (timer modulo/reload value)
- TAC register (timer control)

## Step 1: Create the Timer Module

Create a new file: `src/timer/mod.rs`

### Module Structure
Your timer module should have:
```rust
pub struct Timer {
    // Internal cycle counter (16-bit, DIV is upper 8 bits)
    div_counter: u16,

    // Timer registers
    tima: u8,  // Timer counter (0xFF05)
    tma: u8,   // Timer modulo (0xFF06)
    tac: u8,   // Timer control (0xFF07)

    // Cycle tracking for TIMA
    tima_counter: u16,
}
```

### Required Methods
1. `new()` - Constructor
2. `tick(cycles: u8)` - Update timer by given cycles, returns true if interrupt should fire
3. `read_register(address: u16) -> u8` - Read timer register
4. `write_register(address: u16, value: u8)` - Write timer register

## Step 2: Implement DIV Register

### DIV Behavior
- Increments automatically every 256 CPU cycles
- Read from 0xFF04 returns upper 8 bits of internal counter
- Write to 0xFF04 resets internal counter to 0

### Implementation Hints
- Use a 16-bit `div_counter` that increments every cycle
- `DIV = (div_counter >> 8) as u8`
- On write to 0xFF04: `div_counter = 0`

## Step 3: Implement TAC Register

### TAC Format
```
Bit 2: Timer Enable (0 = disabled, 1 = enabled)
Bits 1-0: Clock Select
  00 = 1024 cycles per TIMA increment (4096 Hz)
  01 = 16 cycles per TIMA increment (262144 Hz)
  10 = 64 cycles per TIMA increment (65536 Hz)
  11 = 256 cycles per TIMA increment (16384 Hz)
```

### Implementation Hints
- Store TAC value (only lower 3 bits are used)
- Create helper method: `fn is_timer_enabled() -> bool`
- Create helper method: `fn get_tima_frequency() -> u16` (returns cycles per increment)

## Step 4: Implement TIMA/TMA Registers

### TIMA Behavior
- Only increments when timer is enabled (TAC bit 2 = 1)
- Increments based on frequency set in TAC bits 1-0
- On overflow (0xFF → 0x00):
  1. Load value from TMA into TIMA
  2. Return true to signal timer interrupt

### Implementation Hints
- Track cycles accumulated in `tima_counter`
- When `tima_counter >= frequency`, increment TIMA and subtract frequency from counter
- Check for overflow: if TIMA was 0xFF and got incremented
- On overflow: `tima = tma` and return interrupt flag

## Step 5: Implement tick() Method

### Pseudocode
```
fn tick(cycles: u8) -> bool:
    interrupt_requested = false

    // Update DIV (always running)
    div_counter += cycles

    // Update TIMA (only if enabled)
    if timer_enabled:
        tima_counter += cycles
        frequency = get_tima_frequency()

        while tima_counter >= frequency:
            tima_counter -= frequency
            old_tima = tima
            tima = tima.wrapping_add(1)

            // Check for overflow
            if old_tima == 0xFF and tima == 0x00:
                tima = tma
                interrupt_requested = true

    return interrupt_requested
```

## Step 6: Integrate with Memory System

Update `src/memory/mod.rs` to:
1. Add a `timer: Timer` field
2. Route reads/writes for 0xFF04-0xFF07 to timer module

### Memory Integration
```rust
// In Memory::read_byte()
match address {
    0xFF04..=0xFF07 => self.timer.read_register(address),
    // ... rest of cases
}

// In Memory::write_byte()
match address {
    0xFF04..=0xFF07 => self.timer.write_register(address, value),
    // ... rest of cases
}
```

## Step 7: Update GameBoy Main Loop

The `step()` method in GameBoy needs to:
1. Execute CPU instruction (returns cycles)
2. Update timer with those cycles
3. Handle timer interrupt if requested

### Example
```rust
pub fn step(&mut self) {
    let cycles = self.cpu.execute(&mut self.memory);

    // Update timer and check for interrupt
    let timer_interrupt = self.memory.timer.tick(cycles);
    if timer_interrupt {
        // Set bit 2 of IF register (0xFF0F)
        // TODO: Implement interrupt system
    }
}
```

## Step 8: Testing

Create tests for:
1. **DIV increment**: Run 256 cycles, check DIV incremented by 1
2. **DIV reset**: Write to DIV, verify it resets to 0
3. **TIMA disabled**: Set TAC bit 2 = 0, verify TIMA doesn't increment
4. **TIMA increment**: Enable timer, run appropriate cycles, verify TIMA increments
5. **TIMA overflow**: Set TIMA = 0xFF, TMA = 0x50, increment once, verify TIMA = 0x50 and interrupt fires
6. **Different frequencies**: Test all 4 TAC frequency settings

### Example Test Structure
```rust
#[test]
fn test_div_increments() {
    let mut gb = GameBoy::new();
    let initial_div = gb.memory.timer.read_register(0xFF04);

    // Run 256 cycles
    for _ in 0..256 {
        gb.memory.timer.tick(1);
    }

    let new_div = gb.memory.timer.read_register(0xFF04);
    assert_eq!(new_div, initial_div.wrapping_add(1));
}
```

## Step 9: Interrupt Integration (Future)

For now, you can just set a flag when TIMA overflows. Later, when implementing the interrupt system:
- Write to IF register (0xFF0F) bit 2 when timer interrupt fires
- If interrupts enabled, CPU will jump to 0x0050

## Reference Values

### TAC Frequency Table
```rust
fn get_tima_frequency(&self) -> u16 {
    match self.tac & 0x03 {
        0 => 1024,  // 00: 4096 Hz
        1 => 16,    // 01: 262144 Hz
        2 => 64,    // 10: 65536 Hz
        3 => 256,   // 11: 16384 Hz
        _ => unreachable!(),
    }
}
```

### Initial Values
```
DIV: 0xAB (varies, can start at any value)
TIMA: 0x00
TMA: 0x00
TAC: 0xF8 (timer disabled)
```

## Debugging Tips

1. **DIV not incrementing**: Check if you're accumulating cycles correctly
2. **TIMA incrementing too fast/slow**: Verify frequency calculation from TAC
3. **TIMA not incrementing**: Check if timer is enabled (TAC bit 2)
4. **Overflow not working**: Ensure you're checking for 0xFF → 0x00 transition

## Next Steps

After implementing the timer:
1. Test with simple programs that use the timer
2. Implement interrupt system to handle timer interrupts
3. Test with real Game Boy ROMs that rely on timing

## Resources

- Reference document: `docs/TIMER_REFERENCE.md`
- Pan Docs: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
- Test with Blargg's timer test ROMs