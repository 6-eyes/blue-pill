mod reg_map;

use embedded_hal::{delay, i2c::{I2c, SevenBitAddress}};

#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2c(E),
    /// Invalid chip id
    InvalidChipId(u8),
}

#[derive(Debug)]
pub struct Bno055<I> {
    i2c: I,
    i2c_addr: u8,
    operation_mode: OperationMode,
    page: RegisterPage,
    temp_source: TempSource,
}

impl<I, E> Bno055<I>
where
    I: I2c<SevenBitAddress, Error = E>,
{
    pub fn new(i2c: I) -> Self {
        Self {
            i2c,
            i2c_addr: crate::bno055::reg_map::DEFAULT_ADDR,
            operation_mode: OperationMode::default(),
            page: RegisterPage::Page0,
            temp_source: TempSource::default(),
        }
    }

    pub fn with_alternate_addr(mut self) -> Self {
        self.i2c_addr = reg_map::ALTERNATE_ADDR;
        self
    }

    pub fn init(&mut self, delay: &mut dyn delay::DelayNs) -> Result<(), Error<E>> {
        // check chip id
        self.set_page(RegisterPage::Page0)?;
        let id = self.read_u8(reg_map::CHIP_ID)?;

        if id != self.i2c_addr {
            Err(Error::InvalidChipId(id))
        }
        else {
            // perform soft reset
            // Reference Headings: 3.2, 4.3.63 
            self.write_u8(reg_map::SYS_TRIGGER, 1u8 << 5)?;
            delay.delay_ms(650);

            // set config mode
            self.set_mode(OperationMode::Config, delay)?;

            // set power mode
            self.write_u8(reg_map::PWR_MODE, PowerMode::Normal as u8)?;

            // reset SYS_TRIGGER register for self test
            self.write_u8(reg_map::SYS_TRIGGER, 0)?;

            Ok(())
        }
    }

    pub fn set_external_oscillator(&mut self, ext: bool, delay: &mut dyn delay::DelayNs) -> Result<(), Error<E>> {
        // capture current mode
        let mode = self.operation_mode;
        // set config mode
        self.set_mode(OperationMode::Config, delay)?;
        // write to the SYS_TRIGGER register
        self.write_u8(reg_map::SYS_TRIGGER, match ext {
            true => 1 << 7,
            false => 0,
        })?;

        self.set_mode(mode, delay)?;
        Ok(())
    }

    pub fn set_units(&mut self, unit: Unit, delay: &mut dyn delay::DelayNs) -> Result<(), Error<E>> {
        let mode = self.operation_mode;
        self.set_mode(OperationMode::Config, delay)?;
        self.write_u8(reg_map::UNIT_SEL, unit.byte())?;
        self.set_mode(mode, delay)?;

        Ok(())
    }

    pub fn set_mode(&mut self, mode: OperationMode, delay: &mut dyn delay::DelayNs) -> Result<(), Error<E>> {
        if self.operation_mode != mode {
            self.set_page(RegisterPage::Page0)?;
            self.write_u8(reg_map::OPR_MODE, mode as u8)?;

            delay.delay_ms(match self.operation_mode {
                OperationMode::Config => 7,
                _ => 19,
            });

            self.operation_mode = mode;
        }

        Ok(())
    }

    pub fn set_temp_source(&mut self, source: TempSource, delay: &mut dyn delay::DelayNs) -> Result<(), Error<E>> {
        if self.temp_source != source {
            let mode = self.operation_mode;
            self.set_mode(OperationMode::Config, delay)?;
            self.write_u8(reg_map::TEMP_SOURCE, source as u8)?;

            self.set_mode(mode, delay)?;
        }

        Ok(())
    }

    pub fn temperature(&mut self) -> Result<i8, Error<E>> {
        self.read_u8(reg_map::TEMP).map(|t| t as i8)
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
        match self.page == page {
            true => Ok(()),
            false => self.write_u8(reg_map::PAGE_ID, page as u8).map(|_| self.page = page),
        }
    }
}

#[derive(Debug, Default)]
pub enum PowerMode {
    #[default]
    Normal = 0b00,
    LowPower = 0b01,
    Suspend = 0b10,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum OperationMode {
    #[default]
    Config = 0b0000,
    Acc = 0b0001,
    Mag = 0b0010,
    Gyro = 0b0011,
    AccMag = 0b0100,
    Accgyro = 0b0101,
    MagGyro = 0b0110,
    Amg = 0b0111,
    Imu = 0b1000,
    Compass = 0b1001,
    M4g = 0b1010,
    NdofFmcOff = 0b1011,
    Ndof = 0b1100,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum RegisterPage {
    Page0 = 0,
    Page1 = 1,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum TempSource {
    #[default]
    Accelerometer = 0b00,
    Gyroscope = 0b01,
}

#[derive(Debug, Default)]
pub struct Unit {
    orientation: Orientation,
    temperature: TempUnit,
    euler: EulerUnit,
    gyro: GyroUnit,
    acc: AccUnit,
}

impl Unit {
    fn byte(&self) -> u8 {
        let mut byte = 0;
        byte |= self.acc as u8;
        byte |= (self.gyro as u8) << 1;
        byte |= (self.euler as u8) << 2;
        byte |= (self.temperature as u8) << 4;
        byte |= (self.orientation as u8) << 7;

        byte
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TempUnit {
    #[default]
    Celcius = 0,
    Fahrenheit = 1,
}

#[derive(Debug, Default, Clone, Copy)]
enum Orientation {
    Android = 1,
    #[default]
    Windows = 0,
}

#[derive(Debug, Default, Clone, Copy)]
enum EulerUnit {
    #[default]
    Degrees = 0,
    Radians = 1,
}

#[derive(Debug, Default, Clone, Copy)]
enum GyroUnit {
    #[default]
    DegreesPerSecond = 0,
    RadiansPerSecons = 1,
}

#[derive(Debug, Default, Clone, Copy)]
enum AccUnit {
    #[default]
    MetersPerSecond = 0,
    MilliG = 1,
}