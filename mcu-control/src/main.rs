#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, serial::{Config, Serial}, timer::Timer};

#[entry]
fn main() -> ! {
    // core peripherals
    let cp = pac::CorePeripherals::take().unwrap();
    // device peripherals
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();

    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(72.MHz()).hclk(8.MHz()).freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut gpioa = dp.GPIOA.split();
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;
    let mut serial = Serial::new(dp.USART1, (tx, rx), &mut afio.mapr, Config::default().baudrate(115200.bps()), &clocks);

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    loop {
        nb::block!(timer.wait()).unwrap();
        for byte in 20u32.to_le_bytes() {
            nb::block!(serial.tx.write(byte));
        }
        led.toggle();
    }
}