# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Game Boy emulator written in Rust. The CPU instruction set is 100% complete (all 446 opcodes implemented: 190 main + 256 CB-prefixed). The next phase is implementing hardware features (timer, interrupts, PPU).

## Build and Test Commands

```bash
# Build the project
cargo build

# Run tests (118 tests covering all CPU instructions)
cargo test

# Run a specific test
cargo test test_name

# Run with clippy (pedantic mode enabled)
cargo clippy

# Run the emulator with a ROM
cargo run <rom_file> [log_file]
# Example: cargo run test_roms/cpu_instrs.test test.txt
```

## Architecture

### Module Structure

```
src/
├── main.rs          - CLI entry point, GameBoy struct integration, 118 tests
├── cpu/
│   ├── mod.rs       - CPU struct, all instruction implementations
│   ├── registers.rs - Register structures (8-bit, 16-bit pairs, flags)
│   └── instructions.rs - Opcode dispatcher (execute() function)
├── memory/
│   └── mod.rs       - 64KB address space, cartridge integration
├── cartridge/
│   └── mod.rs       - ROM loading, MBC1 implementation, header parsing
└── gameboy/
    └── mod.rs       - Main GameBoy struct, ties CPU + Memory together
```

### Key Design Patterns

**CPU and Memory Separation**: CPU and Memory are separate structs. Instructions take `&mut Memory` as a parameter, not `&mut self.memory`.

**Opcode Dispatch**: The main dispatch happens in `cpu/instructions.rs::execute_opcode()`. It's a large match statement mapping opcodes (0x00-0xFF) to instruction implementations.

**CB-Prefixed Instructions**: Opcode 0xCB triggers a second fetch and dispatch for the 256 extended bit manipulation instructions.

**Register Pairs**: 8-bit registers (A, B, C, D, E, H, L) can be accessed as 16-bit pairs (AF, BC, DE, HL) via helper methods in `registers.rs`.

**Flag Register**: The F register is a struct with boolean fields (z, n, h, c) and conversion methods to/from u8.

**PC Increment**: PC increments happen inside `fetch_byte()` and `fetch_word()`, not in individual instruction implementations.

**Cycle Counting**: Every instruction returns a u8 representing CPU cycles consumed (4-24 cycles typical).

### Memory Map Integration

Memory reads/writes are routed based on address ranges:
- `0x0000-0x7FFF`: Cartridge ROM (routed to cartridge module)
- `0x8000-0x9FFF`: Video RAM (internal memory)
- `0xA000-0xBFFF`: Cartridge RAM (routed to cartridge module)
- `0xC000-0xDFFF`: Work RAM (internal memory)
- `0xE000-0xFDFF`: Echo RAM (mirror of Work RAM)
- `0xFE00-0xFE9F`: OAM (Object Attribute Memory)
- `0xFF00-0xFF7F`: I/O Registers
- `0xFF80-0xFFFE`: High RAM (HRAM)
- `0xFFFF`: Interrupt Enable register

### Cartridge System

Cartridges are loaded separately and integrated into Memory via `load_cartridge()`. The cartridge handles:
- ROM banking (MBC1: switchable 16KB banks)
- RAM banking (MBC1: switchable 8KB banks)
- Bank switching via writes to ROM address space

### GameBoy-Doctor Logging

The emulator can output CPU state logs in gameboy-doctor format for validation:
- Format: `A:XX F:XX B:XX C:XX D:XX E:XX H:XX L:XX SP:XXXX PC:XXXX PCMEM:XX,XX,XX,XX`
- Enable via second CLI argument
- Logged before each instruction execution

## Implementation Status

**Complete:**
- All 446 CPU instructions (100% of Game Boy instruction set)
- Cartridge loading and MBC1 memory bank controller
- CPU state logging for validation
- 118 comprehensive tests

**Not Yet Implemented:**
- Timer system (DIV, TIMA, TMA, TAC registers) - see `docs/TIMER_IMPLEMENTATION_GUIDE.md`
- Interrupt handling (VBlank, LCD, Timer, Serial, Joypad)
- PPU (Picture Processing Unit) - graphics rendering
- LCD registers (LCDC, STAT, LY, etc.)
- Joypad input
- Serial port
- Sound (APU)
- MBC3, MBC5 support (only MBC1 implemented)

## Adding New Instructions

When adding instructions (not needed - CPU is complete):
1. Add opcode case in `cpu/instructions.rs::execute_opcode()`
2. Implement function in `cpu/mod.rs`
3. Add test in `src/main.rs` test module
4. Return correct cycle count

## Adding Hardware Features

**Current Priority: Timer System**

See `docs/TIMER_IMPLEMENTATION_GUIDE.md` for step-by-step guide.

General pattern for hardware features:
1. Create new module in `src/<feature>/mod.rs`
2. Integrate with Memory system for I/O register access (0xFF00-0xFF7F)
3. Update `GameBoy::step()` to tick the new component with CPU cycles
4. Handle any interrupt requests (will need interrupt system first)

## Testing Strategy

Tests are in `src/main.rs` under `#[cfg(test)]`. Each test:
1. Creates a GameBoy instance
2. Writes opcodes/data directly to memory
3. Calls `cpu.execute()`
4. Asserts register/memory state and cycle count

For hardware features, test timing-critical behavior (e.g., timer overflow at exact cycle count).

## Reference Documentation

- `PROJECT.md` - Detailed project state, completed features, next steps
- `docs/TIMER_REFERENCE.md` - How the Game Boy timer hardware works
- `docs/TIMER_IMPLEMENTATION_GUIDE.md` - Step-by-step implementation guide
- `The Cycle-Accurate Game Boy Docs (1).pdf` - Hardware reference (in project root)
- Pan Docs: https://gbdev.io/pandocs/

## Clippy Configuration

Pedantic mode is enabled with specific exceptions:
- `must_use_candidate` - allowed
- `verbose_bit_mask` - allowed
- `unused_self` - allowed