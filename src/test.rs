#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{pac, prelude::*};
use embedded_hal::blocking::i2c::Write;

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

    delay.delay_ms(500u32);

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

    // 5 parpadeos rápidos = I2C inicializado, arranca scan
    for _ in 0..5 {
        led.set_low();
        delay.delay_ms(50u32);
        led.set_high();
        delay.delay_ms(50u32);
    }

    delay.delay_ms(500u32);

    // Escanear todas las direcciones I2C
    let mut encontrada: u8 = 0;
    for addr in 0x03u8..=0x77u8 {
        if i2c.write(addr, &[0x00]).is_ok() {
            encontrada = addr;
            break;
        }
        delay.delay_ms(10u32);
    }

    // 5 parpadeos rápidos = scan terminó
    for _ in 0..5 {
        led.set_low();
        delay.delay_ms(50u32);
        led.set_high();
        delay.delay_ms(50u32);
    }

    delay.delay_ms(1000u32);

    // Mostrar resultado
    loop {
        if encontrada > 0 {
            // 0x27 → 7 parpadeos
            // 0x3F → 15 parpadeos
            let veces = encontrada & 0x0F;
            for _ in 0..veces {
                led.set_low();
                delay.delay_ms(300u32);
                led.set_high();
                delay.delay_ms(300u32);
            }
        } else {
            // 1 parpadeo largo = no encontró nada
            led.set_low();
            delay.delay_ms(1000u32);
            led.set_high();
            delay.delay_ms(1000u32);
        }
        delay.delay_ms(2000u32);
    }
}
