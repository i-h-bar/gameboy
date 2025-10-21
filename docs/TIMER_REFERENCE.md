# Game Boy Timer System Reference

## Overview
The Game Boy has a timer system controlled by 4 registers in the I/O region (0xFF00-0xFF7F).

## Timer Registers

### 1. DIV (0xFF04) - Divider Register
- **Read/Write**: Read-only (writes reset it to 0x00)
- **Function**: Free-running counter that increments at 16384 Hz (every 256 CPU cycles)
- **Uses**: Basic timing, random number generation
- **Behavior**: Always running, cannot be stopped
- Upper 8 bits of an internal 16-bit counter that increments every CPU cycle

### 2. TIMA (0xFF05) - Timer Counter
- **Read/Write**: Read/Write
- **Function**: Programmable timer that increments at a configurable frequency
- **Behavior**:
  - Increments based on TAC frequency setting
  - When it overflows (0xFF → 0x00), it:
    1. Loads the value from TMA
    2. Triggers a Timer interrupt (IF bit 2 set to 1)

### 3. TMA (0xFF06) - Timer Modulo
- **Read/Write**: Read/Write
- **Function**: The value that gets loaded into TIMA when TIMA overflows
- **Use**: Allows you to control how often timer interrupts fire
- **Example**: If TMA=0xF0, TIMA will count from 0xF0 to 0xFF (16 increments) before overflowing

### 4. TAC (0xFF07) - Timer Control
- **Read/Write**: Read/Write
- **Format**: `---- -E SS`
  - **Bit 2 (E)**: Timer Enable (1 = enabled, 0 = disabled)
  - **Bits 1-0 (SS)**: Clock Select (frequency)

**Clock Select Frequencies:**
```
Bits 1-0 | Frequency  | CPU Cycles per Increment
---------|------------|-------------------------
   00    |  4096 Hz   | 1024 cycles
   01    | 262144 Hz  | 16 cycles
   10    | 65536 Hz   | 64 cycles
   11    | 16384 Hz   | 256 cycles
```

## How It Works Together

### DIV Register:
```
CPU running → Internal counter increments every cycle
            → DIV = upper 8 bits of internal counter
            → Increments at 16384 Hz (every 256 CPU cycles)
            → Writing any value to DIV resets it to 0x00
```

### TIMA/TMA Timer:
```
1. TAC bit 2 = 1 (timer enabled)
2. TIMA increments at rate determined by TAC bits 1-0
3. When TIMA overflows (0xFF → 0x00):
   a. TIMA ← TMA
   b. Set bit 2 of IF register (0xFF0F) to request interrupt
4. If interrupts enabled, CPU jumps to 0x0050 (timer interrupt handler)
```

## Example Usage

**Game wants 60 Hz timer interrupt:**
- CPU speed: 4,194,304 Hz
- Target: 60 Hz = 69,905 cycles per interrupt
- Closest TAC setting: 00 (4096 Hz, 1024 cycles per increment)
- Need ~68.3 increments: 256 - 68 = 188 (0xBC)

```
TMA ← 0xBC  // Start counting from 188
TIMA ← 0xBC // Initialize TIMA
TAC ← 0x04  // Enable timer, 4096 Hz (bits 1-0 = 00)
// TIMA will count: BC, BD, BE, ..., FF, overflow → interrupt
// Fires every ~68 increments = ~60 Hz
```

## Memory Addresses Summary

```
0xFF04: DIV  - Divider (read-only, write=reset)
0xFF05: TIMA - Timer Counter (read/write)
0xFF06: TMA  - Timer Modulo (read/write)
0xFF07: TAC  - Timer Control (read/write)
0xFF0F: IF   - Interrupt Flags (bit 2 = timer interrupt)
```

## Implementation Requirements

For the emulator, these components are needed:
1. **Track CPU cycles** - Accumulate cycles from each instruction
2. **Update DIV** - Increment every 256 cycles
3. **Update TIMA** - Increment based on TAC frequency when enabled
4. **Handle overflow** - Load TMA into TIMA and set interrupt flag
5. **Handle writes** - DIV writes reset to 0, TIMA/TMA/TAC are writable

## Technical Details

### Internal Counter for DIV
The Game Boy uses a 16-bit internal counter:
- Increments every CPU cycle
- DIV register returns the upper 8 bits
- Writing to DIV resets the entire 16-bit counter to 0

### TIMA Frequency Selection
The TIMA counter actually monitors specific bits of the internal 16-bit counter:
- TAC bits 1-0 = 00: Bit 9 of internal counter (1024 cycles)
- TAC bits 1-0 = 01: Bit 3 of internal counter (16 cycles)
- TAC bits 1-0 = 10: Bit 5 of internal counter (64 cycles)
- TAC bits 1-0 = 11: Bit 7 of internal counter (256 cycles)

When the monitored bit falls from 1 to 0 AND timer is enabled, TIMA increments.

### Overflow Behavior
When TIMA overflows from 0xFF to 0x00:
1. TIMA is loaded with the value from TMA (happens in the next cycle)
2. Timer interrupt flag (IF bit 2) is set
3. If interrupt master enable (IME) is set and timer interrupt is enabled (IE bit 2), CPU jumps to 0x0050

## References
- Pan Docs: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
- The Cycle-Accurate Game Boy Docs