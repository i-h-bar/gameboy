pub struct Timer {
    div_counter: u16,
    tima: u8,
    tma: u8,
    tac: u8, // byte format ---- -E SS; E = Timer enabled, SS is clock frequency
    tima_counter: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tima_counter: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.div_counter = self.div_counter.wrapping_add(cycles as u16);

        true
    }

    pub fn read_register(&self, address: u16) -> u8 {
        // 0xFF04 = DIV, 0xFF05 = TIMA, 0xFF06 = TMA, 0xFF07 = TAC
        match address {
            0xFF04 => (self.div_counter >> 8) as u8,
            0xFF07 => self.tac,
            _ => todo!(),
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        // 0xFF04 = DIV, 0xFF05 = TIMA, 0xFF06 = TMA, 0xFF07 = TAC
        match address {
            0xFF04 => self.div_counter = 0,
            0xFF07 => self.tac = value,
            _ => todo!()
        }
    }

    fn is_timer_enabled(&self) -> bool {
        self.tac & 4 != 0 // 4 = 0b0000100
    }

    fn get_tima_frequency(&self) -> u16 {
        match self.tac & 3 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!("Bottom 2 nibbles was greater than 3?"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod div {
        use super::*;

        #[test]
        fn starts_at_zero() {
            let timer = Timer::new();
            let div = timer.read_register(0xFF04);
            assert_eq!(div, 0x00, "DIV should start at 0x00");
        }

        #[test]
        fn no_increment_after_255_cycles() {
            let mut timer = Timer::new();
            let initial_div = timer.read_register(0xFF04);

            for _ in 0..255 {
                timer.tick(1);
            }

            let new_div = timer.read_register(0xFF04);
            assert_eq!(
                new_div, initial_div,
                "DIV should not increment after 255 cycles"
            );
        }

        #[test]
        fn increments_after_256_cycles() {
            let mut timer = Timer::new();
            let initial_div = timer.read_register(0xFF04);

            for _ in 0..256 {
                timer.tick(1);
            }

            let new_div = timer.read_register(0xFF04);
            assert_eq!(
                new_div,
                initial_div.wrapping_add(1),
                "DIV should increment by 1 after 256 cycles"
            );
        }

        #[test]
        fn increments_after_511_cycles() {
            let mut timer = Timer::new();
            let initial_div = timer.read_register(0xFF04);

            for _ in 0..511 {
                timer.tick(1);
            }

            let new_div = timer.read_register(0xFF04);
            assert_eq!(
                new_div,
                initial_div.wrapping_add(1),
                "DIV should increment by 1 after 511 cycles"
            );
        }

        #[test]
        fn increments_twice_after_512_cycles() {
            let mut timer = Timer::new();
            let initial_div = timer.read_register(0xFF04);

            for _ in 0..512 {
                timer.tick(1);
            }

            let new_div = timer.read_register(0xFF04);
            assert_eq!(
                new_div,
                initial_div.wrapping_add(2),
                "DIV should increment by 2 after 512 cycles"
            );
        }

        #[test]
        fn wraps_around() {
            let mut timer = Timer::new();

            // Tick to 0xFF
            for _ in 0..(256 * 255) {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF04), 0xFF);

            // One more increment should wrap to 0
            for _ in 0..256 {
                timer.tick(1);
            }
            assert_eq!(
                timer.read_register(0xFF04),
                0x00,
                "DIV should wrap from 0xFF to 0x00"
            );
            assert_eq!(timer.div_counter, 0, "div_counter should wrap to 0");
        }

        #[test]
        fn reset_on_write() {
            let mut timer = Timer::new();

            // Increment DIV first
            for _ in 0..1024 {
                timer.tick(1);
            }
            assert_ne!(timer.read_register(0xFF04), 0x00, "DIV should not be 0");

            // Write any value to DIV should reset it to 0
            timer.write_register(0xFF04, 0xFF);
            assert_eq!(
                timer.read_register(0xFF04),
                0x00,
                "Writing to DIV should reset it to 0"
            );

            // Verify it starts counting again from 0
            for _ in 0..256 {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF04), 0x01);
        }

        #[test]
        fn increments_with_variable_cycles() {
            let mut timer = Timer::new();
            timer.write_register(0xFF04, 0);

            // Tick with varying cycle counts that sum to 256
            timer.tick(100);
            timer.tick(50);
            timer.tick(106);

            assert_eq!(
                timer.read_register(0xFF04),
                0x01,
                "DIV should increment after 256 total cycles"
            );
        }

        #[test]
        fn unaffected_by_tac() {
            let mut timer = Timer::new();
            timer.write_register(0xFF04, 0x00);

            // Disable timer
            timer.write_register(0xFF07, 0x00);

            // DIV should still increment
            for _ in 0..256 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF04),
                0x01,
                "DIV should increment regardless of TAC"
            );
        }
    }

    mod tac {
        use super::*;

        #[test]
        fn write_and_read() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            assert_eq!(
                timer.read_register(0xFF07) & 0x07,
                0x05,
                "TAC should store lower 3 bits"
            );
        }

        #[test]
        fn only_lower_3_bits_used() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0xFF);
            let tac = timer.read_register(0xFF07);
            assert_eq!(
                tac & 0x07,
                0x07,
                "TAC should only use lower 3 bits for functionality"
            );
        }

        #[test]
        fn is_timer_enabled_when_bit_2_set() {
            let mut timer = Timer::new();

            // Bit 2 = 0 (disabled)
            timer.write_register(0xFF07, 0x00);
            assert!(!timer.is_timer_enabled(), "Timer should be disabled when bit 2 = 0");

            timer.write_register(0xFF07, 0x03);
            assert!(!timer.is_timer_enabled(), "Timer should be disabled when bit 2 = 0");

            // Bit 2 = 1 (enabled)
            timer.write_register(0xFF07, 0x04);
            assert!(timer.is_timer_enabled(), "Timer should be enabled when bit 2 = 1");

            timer.write_register(0xFF07, 0x07);
            assert!(timer.is_timer_enabled(), "Timer should be enabled when bit 2 = 1");

            timer.write_register(0xFF07, 0x05);
            assert!(timer.is_timer_enabled(), "Timer should be enabled when bit 2 = 1");
        }

        #[test]
        fn get_tima_frequency_00() {
            let mut timer = Timer::new();

            // Frequency 00 = 1024 cycles
            timer.write_register(0xFF07, 0x04); // 0b100 (enabled, freq 00)
            assert_eq!(
                timer.get_tima_frequency(),
                1024,
                "Frequency 00 should be 1024 cycles"
            );
        }

        #[test]
        fn get_tima_frequency_01() {
            let mut timer = Timer::new();

            // Frequency 01 = 16 cycles
            timer.write_register(0xFF07, 0x05); // 0b101 (enabled, freq 01)
            assert_eq!(
                timer.get_tima_frequency(),
                16,
                "Frequency 01 should be 16 cycles"
            );
        }

        #[test]
        fn get_tima_frequency_10() {
            let mut timer = Timer::new();

            // Frequency 10 = 64 cycles
            timer.write_register(0xFF07, 0x06); // 0b110 (enabled, freq 10)
            assert_eq!(
                timer.get_tima_frequency(),
                64,
                "Frequency 10 should be 64 cycles"
            );
        }

        #[test]
        fn get_tima_frequency_11() {
            let mut timer = Timer::new();

            // Frequency 11 = 256 cycles
            timer.write_register(0xFF07, 0x07); // 0b111 (enabled, freq 11)
            assert_eq!(
                timer.get_tima_frequency(),
                256,
                "Frequency 11 should be 256 cycles"
            );
        }

        #[test]
        fn get_tima_frequency_ignores_enable_bit() {
            let mut timer = Timer::new();

            // Frequency should be determined by bits 0-1 only, not bit 2
            timer.write_register(0xFF07, 0x01); // Disabled, freq 01
            assert_eq!(
                timer.get_tima_frequency(),
                16,
                "Frequency should be 16 regardless of enable bit"
            );

            timer.write_register(0xFF07, 0x02); // Disabled, freq 10
            assert_eq!(
                timer.get_tima_frequency(),
                64,
                "Frequency should be 64 regardless of enable bit"
            );
        }
    }

    mod tima {
        use super::*;

        #[test]
        fn does_not_increment_when_disabled() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x00); // Disable timer
            timer.write_register(0xFF05, 0x00);

            for _ in 0..10000 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x00,
                "TIMA should not increment when timer is disabled"
            );
        }

        #[test]
        fn increments_with_frequency_1024() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x04); // Enable, frequency 1024
            timer.write_register(0xFF05, 0x00);

            for _ in 0..1024 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x01,
                "TIMA should increment once after 1024 cycles"
            );
        }

        #[test]
        fn increments_twice_with_frequency_1024() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x04);
            timer.write_register(0xFF05, 0x00);

            for _ in 0..2048 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x02,
                "TIMA should increment twice after 2048 cycles"
            );
        }

        #[test]
        fn increments_with_frequency_16() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05); // Enable, frequency 16
            timer.write_register(0xFF05, 0x00);

            for _ in 0..16 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x01,
                "TIMA should increment once after 16 cycles"
            );
        }

        #[test]
        fn increments_multiple_times_with_frequency_16() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);

            for _ in 0..160 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x0A,
                "TIMA should increment 10 times after 160 cycles"
            );
        }

        #[test]
        fn increments_with_frequency_64() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x06); // Enable, frequency 64
            timer.write_register(0xFF05, 0x00);

            for _ in 0..64 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x01,
                "TIMA should increment once after 64 cycles"
            );
        }

        #[test]
        fn increments_with_frequency_256() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x07); // Enable, frequency 256
            timer.write_register(0xFF05, 0x00);

            for _ in 0..256 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x01,
                "TIMA should increment once after 256 cycles"
            );
        }

        #[test]
        fn write_during_operation() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x10);

            // Run some cycles
            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF05), 0x11);

            // Manually write TIMA
            timer.write_register(0xFF05, 0x50);
            assert_eq!(timer.read_register(0xFF05), 0x50);

            // Should continue from new value
            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(
                timer.read_register(0xFF05),
                0x51,
                "TIMA should continue from manually written value"
            );
        }
    }

    mod overflow {
        use super::*;

        #[test]
        fn loads_tma() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFF);
            timer.write_register(0xFF06, 0x50);

            for _ in 0..16 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x50,
                "TIMA should load TMA value on overflow"
            );
        }

        #[test]
        fn returns_interrupt_flag() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFF);
            timer.write_register(0xFF06, 0xAB);

            let mut interrupt_fired = false;

            for _ in 0..16 {
                if timer.tick(1) {
                    interrupt_fired = true;
                }
            }

            assert!(
                interrupt_fired,
                "Timer should return true (interrupt) on TIMA overflow"
            );
            assert_eq!(timer.read_register(0xFF05), 0xAB);
        }

        #[test]
        fn with_tma_zero() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFF);
            timer.write_register(0xFF06, 0x00);

            for _ in 0..16 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x00,
                "TIMA should load 0x00 from TMA on overflow"
            );
        }

        #[test]
        fn continues_after_overflow() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFF);
            timer.write_register(0xFF06, 0x10);

            // Overflow
            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF05), 0x10);

            // Continue incrementing
            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(
                timer.read_register(0xFF05),
                0x11,
                "TIMA should continue incrementing after overflow"
            );
        }

        #[test]
        fn multiple_overflows() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFE);
            timer.write_register(0xFF06, 0xFF);

            let mut interrupt_count = 0;

            // Run 48 cycles (3 increments: FE->FF, FF->00, 00->01)
            for _ in 0..48 {
                if timer.tick(1) {
                    interrupt_count += 1;
                }
            }

            assert_eq!(
                interrupt_count, 1,
                "Should fire interrupt once (FF->00 overflow)"
            );
            assert_eq!(timer.read_register(0xFF05), 0x00);
        }
    }

    mod tma {
        use super::*;

        #[test]
        fn write_and_read() {
            let mut timer = Timer::new();

            timer.write_register(0xFF06, 0xAB);
            assert_eq!(
                timer.read_register(0xFF06),
                0xAB,
                "TMA should store written value"
            );
        }

        #[test]
        fn change_affects_next_overflow() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0xFF);
            timer.write_register(0xFF06, 0x12);

            // Change TMA before overflow
            timer.write_register(0xFF06, 0x34);

            for _ in 0..16 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x34,
                "TIMA should load new TMA value on overflow"
            );
        }
    }

    mod enable_disable {
        use super::*;

        #[test]
        fn can_be_disabled_mid_count() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);

            // Run half a cycle
            for _ in 0..8 {
                timer.tick(1);
            }

            // Disable timer
            timer.write_register(0xFF07, 0x01);

            // Run more cycles
            for _ in 0..16 {
                timer.tick(1);
            }

            assert_eq!(
                timer.read_register(0xFF05),
                0x00,
                "TIMA should not increment when disabled"
            );
        }

        #[test]
        fn can_be_re_enabled() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);

            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF05), 0x01);

            // Disable
            timer.write_register(0xFF07, 0x01);

            // Re-enable
            timer.write_register(0xFF07, 0x05);

            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(
                timer.read_register(0xFF05),
                0x02,
                "TIMA should continue incrementing after re-enable"
            );
        }

        #[test]
        fn frequency_change_during_operation() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05); // Frequency 16
            timer.write_register(0xFF05, 0x00);

            for _ in 0..16 {
                timer.tick(1);
            }
            assert_eq!(timer.read_register(0xFF05), 0x01);

            // Change to frequency 64
            timer.write_register(0xFF07, 0x06);

            for _ in 0..64 {
                timer.tick(1);
            }
            assert_eq!(
                timer.read_register(0xFF05),
                0x02,
                "TIMA should increment with new frequency"
            );
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn div_and_tima_independent() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x04); // Frequency 1024
            timer.write_register(0xFF05, 0x00);
            timer.write_register(0xFF04, 0x00);

            // Run 256 cycles (DIV should increment, TIMA should not)
            for _ in 0..256 {
                timer.tick(1);
            }

            assert_eq!(timer.read_register(0xFF04), 0x01, "DIV should increment");
            assert_eq!(
                timer.read_register(0xFF05),
                0x00,
                "TIMA should not increment yet"
            );
        }

        #[test]
        fn stress_many_cycles() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);
            timer.write_register(0xFF06, 0x00);

            let mut total_interrupts = 0;

            // Run 4096 cycles
            for _ in 0..4096 {
                if timer.tick(1) {
                    total_interrupts += 1;
                }
            }

            // 4096 / 16 = 256 increments = 1 overflow (0xFF -> 0x00)
            assert_eq!(
                total_interrupts, 1,
                "Should fire 1 interrupt after 256 increments"
            );
        }

        #[test]
        fn variable_tick_sizes() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);

            timer.tick(4);
            timer.tick(7);
            timer.tick(5); // Total: 16 cycles

            assert_eq!(
                timer.read_register(0xFF05),
                0x01,
                "TIMA should handle variable tick sizes"
            );
        }

        #[test]
        fn large_tick_causes_multiple_increments() {
            let mut timer = Timer::new();

            timer.write_register(0xFF07, 0x05);
            timer.write_register(0xFF05, 0x00);

            // Single tick with 80 cycles (should increment 5 times)
            timer.tick(80);

            assert_eq!(
                timer.read_register(0xFF05),
                0x05,
                "Large tick should cause multiple TIMA increments"
            );
        }
    }
}
