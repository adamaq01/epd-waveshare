//! A simple Driver for the Waveshare 4inch e-Paper HAT+ (E) Display via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/wiki/4inch_e-Paper_HAT+_(E))
//! - [Waveshare C driver](https://github.com/waveshareteam/e-Paper/blob/b304c5151aa1edb31bc35e9eaf660f4dc7769590/E-paper_Separate_Program/4inch_e-Paper_E/RaspberryPi_JetsonNano/c/lib/e-Paper/EPD_4in0e.c)
//! - [Waveshare Python driver](https://github.com/waveshareteam/e-Paper/blob/b304c5151aa1edb31bc35e9eaf660f4dc7769590/E-paper_Separate_Program/4inch_e-Paper_E/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd4in0e.py)

use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};

use self::command::Command;
use crate::{
    buffer_len,
    color::SixColor,
    interface::DisplayInterface,
    traits::{InternalWiAdditions, WaveshareDisplay},
};

mod command;

/// Full size buffer for use with the 7in3f EPD
#[cfg(feature = "graphics")]
pub type Display4in0e = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize * 4) },
    SixColor,
>;

/// Width of the display
pub const WIDTH: u32 = 400;
/// Height of the display
pub const HEIGHT: u32 = 600;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: SixColor = SixColor::White;
/// Default mode of writing data (single byte vs blockwise)
const SINGLE_BYTE_WRITE: bool = true;

/// Epd57n3f driver
pub struct Epd4in0e<SPI, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, DELAY, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: SixColor,
}

impl<SPI, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, BUSY, DC, RST, DELAY>
    for Epd4in0e<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        self.interface.reset(delay, 20_000, 2_000);
        self.wait_busy_low(delay);
        delay.delay_ms(30);

        self.cmd_with_data(spi, Command::CMDH, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18])?;
        self.cmd_with_data(spi, Command::Ox01, &[0x3F])?;
        self.cmd_with_data(spi, Command::Ox00, &[0x5F, 0x69])?;
        self.cmd_with_data(spi, Command::Ox05, &[0x40, 0x1F, 0x1F, 0x2C])?;
        self.cmd_with_data(spi, Command::Ox08, &[0x6F, 0x1F, 0x1F, 0x22])?;
        self.cmd_with_data(spi, Command::Ox06, &[0x6F, 0x1F, 0x17, 0x17])?;
        self.cmd_with_data(spi, Command::Ox03, &[0x00, 0x54, 0x00, 0x44])?;
        self.cmd_with_data(spi, Command::Ox60, &[0x02, 0x00])?;
        self.cmd_with_data(spi, Command::Ox30, &[0x08])?;
        self.cmd_with_data(spi, Command::Ox50, &[0x3F])?;
        self.cmd_with_data(spi, Command::Ox61, &[0x01, 0x90, 0x02, 0x58])?;
        self.cmd_with_data(spi, Command::OxE3, &[0x2F])?;
        self.cmd_with_data(spi, Command::Ox84, &[0x01])
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd4in0e<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = SixColor;

    fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, <SPI>::Error>
    where
        Self: Sized,
    {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd4in0e { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        self.init(spi, delay)
    }

    fn set_background_color(&mut self, color: Self::DisplayColor) {
        self.color = color;
    }

    fn background_color(&self) -> &Self::DisplayColor {
        &self.color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), <SPI>::Error> {
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DataStartTransmission, buffer)
    }

    fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), <SPI>::Error> {
        unimplemented!()
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        self.command(spi, Command::PowerOn)?;
        self.wait_busy_low(delay);
        delay.delay_ms(200);

        self.cmd_with_data(spi, Command::Ox06, &[0x6F, 0x1F, 0x17, 0x27])?;
        delay.delay_ms(200);

        self.cmd_with_data(spi, Command::DataFresh, &[0x00])?;
        self.wait_busy_low(delay);

        self.cmd_with_data(spi, Command::PowerOff, &[0x00])?;
        self.wait_busy_low(delay);

        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), <SPI>::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        let bg = SixColor::colors_byte(self.color, self.color);

        self.wait_busy_low(delay);
        self.command(spi, Command::DataStartTransmission)?;
        self.interface.data_x_times(spi, bg, WIDTH * HEIGHT / 2)?;

        self.display_frame(spi, delay)
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _refresh_rate: Option<crate::traits::RefreshLut>,
    ) -> Result<(), <SPI>::Error> {
        unimplemented!()
    }

    fn wait_until_idle(&mut self, _spi: &mut SPI, delay: &mut DELAY) -> Result<(), <SPI>::Error> {
        self.wait_busy_low(delay);
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> Epd4in0e<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_busy_low(&mut self, delay: &mut DELAY) {
        self.interface.wait_until_idle(delay, true);
    }

    /// Show 7 blocks of color, used for quick testing
    pub fn show_7block(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        let color_7 = [
            SixColor::Black,
            SixColor::Yellow,
            SixColor::Red,
            SixColor::Blue,
            SixColor::Green,
            SixColor::White,
        ];

        self.command(spi, Command::DataStartTransmission)?;
        for color in color_7 {
            for _ in 0..20000 {
                self.interface
                    .data(spi, &[SixColor::colors_byte(*color, *color)])?;
            }
        }

        self.display_frame(spi, delay)
    }
}
