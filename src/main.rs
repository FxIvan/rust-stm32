#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};
use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(72.MHz()).freeze(&mut flash.acr);
    let mut delay = cp.SYST.delay(&clocks);
    let mut afio = dp.AFIO.constrain();

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let i2c = stm32f1xx_hal::i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        stm32f1xx_hal::i2c::Mode::Standard { frequency: 25_000.Hz() },
        clocks,
        10000, 10, 10000, 10000,
    );

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let mut esp = Serial::new(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    let mut lcd = HD44780::new_i2c(i2c, 0x27, &mut delay).unwrap();
    lcd.reset(&mut delay).ok();
    lcd.clear(&mut delay).ok();
    lcd.set_display_mode(hd44780_driver::DisplayMode {
        display: hd44780_driver::Display::On,
        cursor_visibility: hd44780_driver::Cursor::Invisible,
        cursor_blink: hd44780_driver::CursorBlink::Off,
    }, &mut delay).ok();

    lcd.write_str("Listo...", &mut delay).ok();

    let mut buf: [u8; 64] = [0u8; 64];
    let mut buf_len: usize = 0;

    loop {
        if let Ok(byte) = esp.rx.read() {
            if byte == b'\n' {
                // Llegó una línea completa — mostrarla en el LCD
                let end = if buf_len > 0 && buf[buf_len - 1] == b'\r' {
                    buf_len - 1
                } else {
                    buf_len
                };

                if let Ok(texto) = core::str::from_utf8(&buf[..end]) {
                    lcd.clear(&mut delay).ok();
                    lcd.write_str(texto, &mut delay).ok();
                }

                buf_len = 0;
            } else if byte != b'\r' {
                if buf_len < buf.len() {
                    buf[buf_len] = byte;
                    buf_len += 1;
                } else {
                    buf_len = 0;
                }
            }
        }
    }
}