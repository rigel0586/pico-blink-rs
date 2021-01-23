#![no_std]
#![no_main]

use cortex_m_rt::{entry, pre_init};
use panic_halt as _;

#[link_section = ".boot_loader"]
#[no_mangle]
pub static BOOT_LOADER: [u8; 512] = *include_bytes!("boot2_and_reset.bin");

#[pre_init]
unsafe fn pre_init() {
    let sio = &*rp2040_pac::SIO::ptr();

    // If we are core 1, then stop dead and let core 0 do all the work.
    if sio.cpuid.read() != 0u32 {
        loop {
            cortex_m::asm::nop();
        }
    }
}

unsafe fn setup_chip(p: &mut rp2040_pac::Peripherals) {
    // Now reset all the peripherals, except QSPI and XIP (we're using those
    // to execute from external flash!)

    p.RESETS.reset.write(|w| {
        w.adc().set_bit();
        w.busctrl().set_bit();
        w.dma().set_bit();
        w.i2c0().set_bit();
        w.i2c1().set_bit();
        w.io_bank0().set_bit();
        w.io_qspi().clear_bit();
        w.jtag().set_bit();
        w.pads_bank0().set_bit();
        w.pads_qspi().clear_bit();
        w.pio0().set_bit();
        w.pio1().set_bit();
        w.pll_sys().clear_bit();
        w.pll_usb().clear_bit();
        w.pwm().set_bit();
        w.rtc().set_bit();
        w.spi0().set_bit();
        w.spi1().set_bit();
        w.syscfg().set_bit();
        w.sysinfo().set_bit();
        w.tbman().set_bit();
        w.timer().set_bit();
        w.uart0().set_bit();
        w.uart1().set_bit();
        w.usbctrl().set_bit();
        w
    });

    // unreset_block_wait(RESETS_RESET_BITS /* 01ff_ffff */ & ~(
    //         RESETS_RESET_ADC_BITS |
    //         RESETS_RESET_RTC_BITS |
    //         RESETS_RESET_SPI0_BITS |
    //         RESETS_RESET_SPI1_BITS |
    //         RESETS_RESET_UART0_BITS |
    //         RESETS_RESET_UART1_BITS |
    //         RESETS_RESET_USBCTRL_BITS
    // ));

    const RESETS_RESET_BITS: u32 = 0x01ffffff;
    const RESETS_RESET_USBCTRL_BITS: u32 = 0x01000000;
    const RESETS_RESET_UART1_BITS: u32 = 0x00800000;
    const RESETS_RESET_UART0_BITS: u32 = 0x00400000;
    const RESETS_RESET_SPI1_BITS: u32 = 0x00020000;
    const RESETS_RESET_SPI0_BITS: u32 = 0x00010000;
    const RESETS_RESET_RTC_BITS: u32 = 0x00008000;
    const RESETS_RESET_ADC_BITS: u32 = 0x00000001;

    const PERIPHERALS_TO_UNRESET: u32 = RESETS_RESET_BITS
        & !(RESETS_RESET_ADC_BITS
            | RESETS_RESET_RTC_BITS
            | RESETS_RESET_SPI0_BITS
            | RESETS_RESET_SPI1_BITS
            | RESETS_RESET_UART0_BITS
            | RESETS_RESET_UART1_BITS
            | RESETS_RESET_USBCTRL_BITS);

    p.RESETS.reset.modify(|_r, w| {
        // w.adc().clear_bit();
        w.busctrl().clear_bit();
        w.dma().clear_bit();
        w.i2c0().clear_bit();
        w.i2c1().clear_bit();
        w.io_bank0().clear_bit();
        w.io_qspi().clear_bit();
        w.jtag().clear_bit();
        w.pads_bank0().clear_bit();
        w.pads_qspi().clear_bit();
        w.pio0().clear_bit();
        w.pio1().clear_bit();
        w.pll_sys().clear_bit();
        w.pll_usb().clear_bit();
        w.pwm().clear_bit();
        // w.rtc().clear_bit();
        // w.spi0().clear_bit();
        // w.spi1().clear_bit();
        w.syscfg().clear_bit();
        w.sysinfo().clear_bit();
        w.tbman().clear_bit();
        w.timer().clear_bit();
        // w.uart0().clear_bit();
        // w.uart1().clear_bit();
        // w.usbctrl().clear_bit();
        w
    });

    while (!p.RESETS.reset_done.read().bits() & PERIPHERALS_TO_UNRESET) != 0 {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let mut p = rp2040_pac::Peripherals::take().unwrap();

    unsafe {
        setup_chip(&mut p);
    }

    // Set GPIO25 to be an input (output enable is cleared)
    p.SIO.gpio_oe_clr.write(|w| unsafe {
        w.bits(1 << 25);
        w
    });

    // Set GPIO25 to be an output low (output is cleared)
    p.SIO.gpio_out_clr.write(|w| unsafe {
        w.bits(1 << 25);
        w
    });

    // Configure pin 25 for GPIO
    p.PADS_BANK0.gpio25.write(|w| {
        // Output Disable off
        w.od().clear_bit();
        // Input Enable on
        w.ie().set_bit();
        w
    });
    p.IO_BANK0.gpio25_ctrl.write(|w| {
        // Map pin 25 to SIO
        w.funcsel().sio_25();
        w
    });

    // Set GPIO25 to be an output (output enable is set)
    p.SIO.gpio_oe_set.write(|w| unsafe {
        w.bits(1 << 25);
        w
    });

    loop {
        for _i in 0..1000000 {
            cortex_m::asm::nop();
        }

        // Set GPIO25 to be low
        p.SIO.gpio_out_clr.write(|w| unsafe {
            w.bits(1 << 25);
            w
        });

        for _i in 0..1000000 {
            cortex_m::asm::nop();
        }

        // Set GPIO25 to be high
        p.SIO.gpio_out_set.write(|w| unsafe {
            w.bits(1 << 25);
            w
        });
    }
}