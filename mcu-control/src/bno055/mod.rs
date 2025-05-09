mod reg_map;

use embedded_hal::i2c::{I2c, SevenBitAddress};

#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2c(E),
}

#[derive(Debug)]
pub struct Bno055<I> {
    i2c: I,
    i2c_addr: u8,
}

impl<I, E> Bno055<I>
where
    I: I2c<SevenBitAddress, Error = E>,
{
    pub fn new(i2c: I) -> Self {
        Self {
            i2c,
            i2c_addr: crate::bno055::reg_map::DEFAULT_ADDR,
        }
    }

    pub fn with_alternate_addr(mut self) -> Self {
        self.i2c_addr = reg_map::ALTERNATE_ADDR;
        self
    }

    fn init(&mut self) -> Result<(), Error<E>> {
        self.set_page(RegisterPage::Page0)?;

        // set temp source
        // set units
        // set operating mode
        todo!()
    }

    fn read_bytes(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), Error<E>> {
        self.i2c.write_read(self.i2c_addr, &[reg], buf).map_err(Error::I2c)
    }

    /// Reads the byte from the register
    fn read_u8(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut byte = [0; 1];
        self.i2c.write_read(self.i2c_addr, &[reg], &mut byte).map(|_| byte[0]).map_err(Error::I2c)
    }

    /// Writes value to the register
    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.i2c.write(self.i2c_addr, &[reg, value]).map_err(Error::I2c)
    }

    /// Sets the register page
    fn set_page(&mut self, page: RegisterPage) -> Result<(), Error<E>> {
        self.write_u8(reg_map::PAGE_ID, page as u8)
    }
}

#[derive(Debug)]
enum RegisterPage {
    Page0 = 0,
    Page1 = 1,
}

mod temp {
    use embedded_hal::i2c::{I2c, SevenBitAddress};

    use crate::bno055::reg_map;

    #[derive(Debug, Default)]
    enum Source {
        #[default]
        Accelerometer = 0b00,
        Gyroscope = 0b01,
    }

    #[derive(Debug, Default)]
    enum Unit {
        #[default]
        Celcius = 0,
        Fahrenheit = 1,
    }

    #[derive(Debug, Default)]
    pub struct Temp {
        source: Source,
        unit: Unit,
    }

    impl Temp {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn set_source(mut self, source: Source) -> Self {
            self.source = source;
            self
        }

        pub fn set_unit(mut self, unit: Unit) -> Self {
            self.unit = unit;
            self
        }
    }

    impl<I, E> super::Bno055<I>
    where
        I: I2c<SevenBitAddress, Error = E>,
    {
        pub fn with_temp(mut self, temp: Temp) -> Self {
            self.temp = temp;
            self
        }

        pub fn temperature(&mut self) -> Result<i8, super::Error<E>> {
            self.read_u8(reg_map::TEMP).map(|t| t as i8)
        }
    }
}