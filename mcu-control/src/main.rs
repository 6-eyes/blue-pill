#![no_std]
#![no_main]

mod bno055;

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f1xx_hal::{i2c::{BlockingI2c, DutyCycle, Mode}, pac::{self, DWT}, prelude::*, serial::{Config, Serial}, timer::Timer};

#[entry]
fn main() -> ! {
    // core peripherals
    let mut cp = pac::CorePeripherals::take().unwrap();
    // device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut afio = dp.AFIO.constrain();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(72.MHz()).hclk(8.MHz()).freeze(&mut flash.acr);

    let mut led = {
        let mut gpioc = dp.GPIOC.split();
        gpioc.pc13.into_push_pull_output(&mut gpioc.crh)
    };

    let (mut tx, _) = {
        let mut gpioa = dp.GPIOA.split();
        let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
        let rx = gpioa.pa10;
        let serial = Serial::new(dp.USART1, (tx, rx), &mut afio.mapr, Config::default().baudrate(115200.bps()), &clocks);
        serial.split()
    };

    DWT::enable_cycle_counter(&mut cp.DWT);
    let i2c = {
        let mut gpiob = dp.GPIOB.split();
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        let mode = Mode::Fast {
            frequency: 400_000.Hz(),
            duty_cycle: DutyCycle::Ratio16to9,
        };

        BlockingI2c::i2c1(dp.I2C1, (scl, sda), &mut afio.mapr, mode, clocks, 1000, 10, 1000, 1000)
    };

    let mut bno055 = bno055::Bno055::new(i2c);

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    loop {
        timer.wait().unwrap();

        for byte in 20i8.to_le_bytes() {
            tx.write(byte).unwrap();
        }
        led.toggle();
    }
}