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

fn uart_read_line(
    rx: &mut impl embedded_hal::serial::Read<u8>,
    buf: &mut [u8],
) -> usize {
    let mut i = 0;

    loop {
        if let Ok(byte) = rx.read() {
            if byte == b'\n' {
                break;
            }

            if i < buf.len() {
                buf[i] = byte;
                i += 1;
            }
        }
    }

    i
}

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
    let mut gpioc = dp.GPIOC.split();

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // I2C LCD
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let i2c = stm32f1xx_hal::i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        stm32f1xx_hal::i2c::Mode::Standard {
            frequency: 25_000.Hz(),
        },
        clocks,
        10000,
        10,
        10000,
        10000,
    );

    // UART ESP8266
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let mut esp = Serial::new(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    // LCD init
    let mut lcd = HD44780::new_i2c(i2c, 0x27, &mut delay).unwrap();
    lcd.reset(&mut delay).ok();
    lcd.clear(&mut delay).ok();
    lcd.set_display_mode(DisplayMode {
        display: Display::On,
        cursor_visibility: Cursor::Invisible,
        cursor_blink: CursorBlink::Off,
    }, &mut delay).ok();

    lcd.write_str("Esperando ESP...", &mut delay).ok();

    delay.delay_ms(3000u32);

    let mut buf: [u8; 64] = [0; 64];

    // ✅ VARIABLES CORRECTAMENTE DENTRO DE MAIN
    let mut ip_line: [u8; 32] = [0; 32];
    let mut ip_len: usize = 0;

    loop {
        let n = uart_read_line(&mut esp.rx, &mut buf);

        if n > 0 {
            if let Ok(texto) = core::str::from_utf8(&buf[..n]) {

                if texto.starts_with("IP:") {
                    lcd.clear(&mut delay).ok();

                    // Línea 1
                    lcd.write_str("IP:", &mut delay).ok();

                    // Línea 2
                    lcd.set_cursor_pos(0x40, &mut delay).ok();

                    let ip = &texto[3..]; // quitar "IP:"
                    lcd.write_str(ip.trim(), &mut delay).ok();
                } else {
                    lcd.clear(&mut delay).ok();
                    lcd.write_str(texto, &mut delay).ok();
                }

                led.set_low();
            }
        }
    }
}