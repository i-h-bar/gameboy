# Game Boy Interrupt System Reference

## Overview
The Game Boy has a sophisticated interrupt system that allows hardware events to temporarily pause normal program execution and jump to specific handler routines. The system consists of 5 interrupt types, controlled by several registers and an internal CPU flag.

## Interrupt Types

The Game Boy supports 5 different interrupt types, each with a specific bit position and vector address:

| Bit | Priority | Name       | Vector | Description                                    |
|-----|----------|------------|--------|------------------------------------------------|
| 0   | 1        | V-Blank    | 0x0040 | Triggered when screen enters V-Blank period    |
| 1   | 2        | LCD STAT   | 0x0048 | LCD controller status interrupt                |
| 2   | 3        | Timer      | 0x0050 | Timer overflow (TIMA register)                 |
| 3   | 4        | Serial     | 0x0058 | Serial transfer completion                     |
| 4   | 5        | Joypad     | 0x0060 | Button press (transition from high to low)     |

**Priority**: Lower bit numbers have higher priority. If multiple interrupts are pending, bit 0 (V-Blank) is serviced first.

## Interrupt Registers

### IF - Interrupt Flag Register (0xFF0F)

**Read/Write**: Read/Write
**Format**: `--- XXXXX` (bits 0-4 used, bits 5-7 unused)

Each bit represents a pending interrupt request:
- **Bit 0**: V-Blank interrupt requested
- **Bit 1**: LCD STAT interrupt requested
- **Bit 2**: Timer interrupt requested
- **Bit 3**: Serial interrupt requested
- **Bit 4**: Joypad interrupt requested

**Behavior**:
- Hardware sets bits when interrupt conditions occur (e.g., timer overflow sets bit 2)
- Software can read this register to check pending interrupts
- Software can write to this register to manually trigger or clear interrupt requests
- When an interrupt is serviced, the CPU does NOT automatically clear the bit - software must clear it in the interrupt handler

### IE - Interrupt Enable Register (0xFFFF)

**Read/Write**: Read/Write
**Format**: `--- XXXXX` (bits 0-4 used, bits 5-7 unused)

Each bit controls whether that interrupt type can trigger:
- **Bit 0**: V-Blank interrupt enable
- **Bit 1**: LCD STAT interrupt enable
- **Bit 2**: Timer interrupt enable
- **Bit 3**: Serial interrupt enable
- **Bit 4**: Joypad interrupt enable

**Behavior**:
- 0 = Interrupt type disabled (won't jump to vector even if IF bit is set)
- 1 = Interrupt type enabled (will jump to vector if IF bit is set AND IME is set)

## IME - Interrupt Master Enable Flag

**Location**: Internal CPU flag (not mapped to memory, cannot be read)
**Values**: 0 = Disabled, 1 = Enabled

**Function**:
- Controls whether the CPU will jump to interrupt vectors
- Does NOT prevent hardware from setting IF bits (interrupts are still requested)
- Only affects whether pending interrupts trigger vector jumps

**Modified by**:
- `EI` instruction: Enables IME (with 1 cycle delay)
- `DI` instruction: Disables IME (immediate)
- `RETI` instruction: Enables IME (immediate)
- Interrupt dispatch: Disables IME automatically when jumping to vector

## Interrupt Processing

### When Does an Interrupt Trigger?

An interrupt is serviced when ALL of these conditions are true:
1. **IME = 1** (Interrupt Master Enable is set)
2. **IF bit is set** (Interrupt has been requested)
3. **IE bit is set** (Interrupt type is enabled)
4. **No higher priority interrupt is pending** (Lower bit number = higher priority)

The condition is: `IME AND (IF AND IE) != 0`

### Interrupt Dispatch Sequence

When an interrupt is serviced, the following happens (takes 20 clock cycles total):

1. **Wait states** (2 machine cycles / 8 clocks): Two internal wait states
2. **Push PC** (2 machine cycles / 8 clocks):
   - SP = SP - 1, write PC_high to (SP)
   - SP = SP - 1, write PC_low to (SP)
3. **Jump to vector** (1 machine cycle / 4 clocks):
   - PC_high = 0x00
   - PC_low = vector address (0x40, 0x48, 0x50, 0x58, or 0x60)
4. **Disable IME**: IME flag is automatically set to 0
5. **Clear IF bit**: Software must do this in the interrupt handler (NOT automatic)

**Special case**: If CPU was halted, dispatch takes 24 clocks (4 extra clocks to wake up).

### Interrupt Priority

If multiple interrupts are pending (multiple bits in IF & IE are set), the CPU services them in priority order:
- Bit 0 (V-Blank) has highest priority
- Bit 4 (Joypad) has lowest priority

Only ONE interrupt is serviced per check. After returning from the interrupt handler (via RETI), the CPU will check again and service the next pending interrupt if any.

## HALT Behavior and Interrupts

The `HALT` instruction stops CPU execution until an interrupt occurs. However, interrupt behavior differs based on IME:

### HALT with IME = 1
- CPU stops executing instructions
- When IF & IE != 0 (interrupt pending):
  - CPU wakes up (4 clocks)
  - Interrupt is dispatched normally (20 more clocks = 24 total)
  - Execution continues at interrupt vector

### HALT with IME = 0
- CPU stops executing instructions
- When IF & IE != 0 (interrupt pending):
  - CPU wakes up and continues at next instruction (4 clocks)
  - Interrupt is NOT dispatched (no vector jump)
  - Software can check IF register to see what woke the CPU

### HALT Bug
There's a hardware bug when HALT is executed with IME = 0 AND no interrupts are pending (IF & IE = 0):
- After waking (when an interrupt eventually gets requested), the byte after HALT is executed twice
- This is typically not emulated in basic emulators

## Interrupt Instructions

### DI - Disable Interrupts (0xF3)
```
DI
```
- **Opcode**: 0xF3
- **Cycles**: 4
- **Effect**: IME = 0 (immediate, takes effect right away)
- **Use**: Disable interrupt dispatch before critical code sections

### EI - Enable Interrupts (0xFB)
```
EI
```
- **Opcode**: 0xFB
- **Cycles**: 4
- **Effect**: IME = 1 (delayed by 1 instruction)
- **Important**: Interrupts are not enabled until AFTER the next instruction executes
- **Use case**: Allows `EI` followed by `RET` to return safely before interrupts trigger

**Example**:
```
EI          ; IME still 0
RET         ; IME still 0, executes safely
; <-- IME becomes 1 here
```

### RETI - Return from Interrupt (0xD9)
```
RETI
```
- **Opcode**: 0xD9
- **Cycles**: 16
- **Effect**:
  - Pop PC from stack (like RET)
  - Enable interrupts (IME = 1, immediate, no delay)
- **Use**: Return from interrupt handler and re-enable interrupts

## Interrupt Handler Pattern

A typical interrupt handler looks like this:

```asm
; Timer interrupt handler at 0x0050
timer_interrupt:
    PUSH AF           ; Save registers
    PUSH BC

    ; Handle the interrupt
    LD A, [$FF05]     ; Read TIMA
    ; ... do work ...

    ; Clear the interrupt flag (REQUIRED!)
    LD A, [$FF0F]     ; Read IF
    AND $FB           ; Clear bit 2 (timer)
    LD [$FF0F], A     ; Write back

    POP BC            ; Restore registers
    POP AF
    RETI              ; Return and re-enable interrupts
```

**Critical points**:
1. Save any registers you use
2. Clear the IF bit for this interrupt (hardware doesn't do this automatically)
3. Restore registers
4. Use RETI (not RET) to re-enable interrupts

## Nested Interrupts

Nested interrupts (interrupt handlers being interrupted) are possible but rare:
- When an interrupt is dispatched, IME is automatically disabled
- If the interrupt handler executes `EI`, interrupts can trigger again
- This allows higher-priority interrupts to interrupt lower-priority handlers
- Stack must be deep enough to handle nested PUSH operations

**Example**:
```asm
timer_interrupt:
    PUSH AF
    EI              ; Re-enable interrupts (allows nesting)
    ; ... long handler code ...
    ; V-Blank could interrupt us here if it's pending
    DI              ; Disable before cleanup
    POP AF
    RETI
```

Most games don't use nested interrupts due to complexity.

## Implementation Gotchas

### EI Delay
The EI instruction enables interrupts with a 1-instruction delay. This is crucial:
- Allows patterns like `EI` followed by `RET` to work safely
- Must track this delay with a flag (e.g., `ei_pending`)
- After executing any instruction with `ei_pending` set, then set IME

### IF Bits Not Auto-Cleared
Unlike some systems, the Game Boy does NOT automatically clear IF bits when servicing interrupts:
- Hardware sets the bit when the interrupt condition occurs
- CPU jumps to the vector if enabled
- **Software must clear the bit** in the interrupt handler
- Forgetting to clear causes infinite interrupt loops

### Interrupt Check Timing
Interrupts should be checked:
- After each instruction execution
- When waking from HALT

Do NOT check during instruction execution (mid-instruction interrupts don't exist on Game Boy).

### HALT Wake-Up
When HALT wakes up due to an interrupt:
- With IME = 1: Dispatch interrupt (24 cycles total: 4 wake + 20 dispatch)
- With IME = 0: Just wake up and continue (4 cycles)

## Example: Timer Interrupt Flow

Let's trace a timer interrupt from start to finish:

1. **Timer overflow occurs**:
   - Timer hardware sets bit 2 of IF register (0xFF0F |= 0x04)

2. **CPU checks interrupts** (after instruction execution):
   - IME = 1? ✓
   - IF bit 2 set? ✓ (0xFF0F & 0x04 = 0x04)
   - IE bit 2 set? ✓ (0xFFFF & 0x04 = 0x04)
   - Higher priority interrupts? Check bits 0-1 of (IF & IE)

3. **Dispatch interrupt** (20 cycles):
   - Wait 2 cycles (8 clocks)
   - Push PC to stack (8 clocks)
   - Jump to 0x0050 (4 clocks)
   - Set IME = 0

4. **Handler executes**:
   - Saves registers
   - Clears IF bit 2: `0xFF0F &= ~0x04`
   - Does work
   - Restores registers
   - Executes RETI

5. **RETI executes** (16 cycles):
   - Pops PC from stack
   - Sets IME = 1
   - Continues execution where interrupted

## Memory Map Summary

```
0x0040: V-Blank interrupt vector
0x0048: LCD STAT interrupt vector
0x0050: Timer interrupt vector
0x0058: Serial interrupt vector
0x0060: Joypad interrupt vector
0xFF0F: IF - Interrupt Flag register
0xFFFF: IE - Interrupt Enable register
```

## References
- The Cycle-Accurate Game Boy Docs (pages 14-17)
- Pan Docs: https://gbdev.io/pandocs/Interrupts.html
- ISSOtm's Game Boy hardware reference
