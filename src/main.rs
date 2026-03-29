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
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // I2C para LCD
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

    // UART para ESP8266
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let mut esp = Serial::new(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    delay.delay_ms(500u32);

    // Inicializar LCD
    let mut lcd = HD44780::new_i2c(i2c, 0x27, &mut delay).unwrap();
    lcd.reset(&mut delay).ok();
    lcd.clear(&mut delay).ok();
    lcd.set_display_mode(DisplayMode {
        display: Display::On,
        cursor_visibility: Cursor::Invisible,
        cursor_blink: CursorBlink::Off,
    }, &mut delay).ok();

    led.set_low();

    // Mostrar que arrancó
    lcd.write_str("Probando ESP...", &mut delay).ok();

    delay.delay_ms(1000u32);

    // Mandar AT al ESP8266
    for byte in b"AT\r\n" {
        nb::block!(esp.tx.write(*byte)).ok();
    }

    // Leer respuesta
    let mut respuesta: [u8; 16] = [b' '; 16];
    let mut i = 0;
    let mut timeout = 0u32;

    while i < 16 && timeout < 100000 {
        if let Ok(byte) = esp.rx.read() {
            if byte != b'\r' && byte != b'\n' {
                if i < 16 {
                    respuesta[i] = byte;
                    i += 1;
                }
            }
        }
        timeout += 1;
    }

    // Mostrar respuesta en LCD
    lcd.clear(&mut delay).ok();

    if respuesta.starts_with(b"AT") || respuesta.contains(&b'O') {
        lcd.write_str("ESP responde OK!", &mut delay).ok();
        lcd.set_cursor_pos(40, &mut delay).ok();
        lcd.write_str("WiFi listo", &mut delay).ok();
    } else {
        lcd.write_str("ESP sin respues.", &mut delay).ok();
        lcd.set_cursor_pos(40, &mut delay).ok();
        lcd.write_str("Revisar cables", &mut delay).ok();
    }

    loop {
        led.set_low();
        delay.delay_ms(500u32);
        led.set_high();
        delay.delay_ms(500u32);
    }
}