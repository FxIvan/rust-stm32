#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{pac, prelude::*};
use lcd_lcm1602_i2c::Lcd;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(72.MHz()).freeze(&mut flash.acr);
    let mut delay = cp.SYST.delay(&clocks);
    let mut afio = dp.AFIO.constrain();

    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // 3 parpadeos = arrancó
    for _ in 0..3 {
        led.set_low();
        delay.delay_ms(200u32);
        led.set_high();
        delay.delay_ms(200u32);
    }

    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let mut i2c = stm32f1xx_hal::i2c::BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        stm32f1xx_hal::i2c::Mode::Standard {
            frequency: 50_000.Hz(),
        },
        clocks,
        10000,
        10,
        10000,
        10000,
    );

    // Esperar que el LCD arranque
    delay.delay_ms(500u32);

    // Inicializar LCD en 0x27
    let mut lcd = Lcd::new(&mut i2c, &mut delay)
        .address(0x27)
        .rows(2)
        .init()
        .unwrap();

    // LED fijo = LCD inicializado OK
    led.set_low();

    // Escribir texto
    lcd.set_cursor(0, 15).ok();
    lcd.write_str("!KO serodivreS").ok();

    lcd.set_cursor(1, 15).ok();
    lcd.write_str("enilno 5/5").ok();

    loop {}
}