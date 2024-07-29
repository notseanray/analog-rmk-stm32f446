#![no_main]
#![no_std]
#![feature(const_option)]

#[macro_use]
mod macros;
mod keymap;
mod vial;

use crate::keymap::{COL, NUM_LAYER, ROW};
use cortex_m::singleton;
use embassy_stm32::{adc::{Adc, RingBufferedAdc, SampleTime, Sequence, VrefInt}, flash::{Blocking, Flash}, gpio::Pin, peripherals::USB_OTG_FS, usb::{Config as UsbConfig, Driver, InterruptHandler}, Peripherals};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{AnyPin, Input, Output},
    Config,
};
use embassy_time::Delay;
use panic_probe as _;
//use panic_halt as _;
//use rmk::{config::{RmkConfig, VialConfig}, embedded_hal::delay::DelayNs, initialize_keyboard_and_run};
use rmk::{config::{RmkConfig, VialConfig}, embedded_hal::delay::DelayNs, initialize_keyboard_and_run};
use static_cell::StaticCell;
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};

bind_interrupts!(struct Irqs {
    OTG_FS => InterruptHandler<USB_OTG_FS>;
});


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // RCC config
    let config = Config::default();
    let mut delay = Delay;

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    // Usb config
    static EP_OUT_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
    let mut usb_config = UsbConfig::default();
    usb_config.vbus_detection = true;
    let driver = Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, &mut EP_OUT_BUFFER.init([0; 1024])[..], usb_config);

    let adc = Adc::new(p.ADC1);
    // stabilize voltage readings
    delay.delay_us(VrefInt::start_time_us());

    // Pin config
    let (input_pins, output_pins) = config_matrix_pins_stm32!(peripherals: p, input: [PD9, PD8, PB13, PB12], output: [PE13, PE14, PE15]);


    // Use internal flash to emulate eeprom
    let f = Flash::new_blocking(p.FLASH);

    // Keyboard config
    let keyboard_config = RmkConfig {
        vial_config: VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF),
        ..Default::default()
    };

    // Start serving
    initialize_keyboard_and_run::<
        Flash<'_, Blocking>,
        Driver<'_, USB_OTG_FS>,
        Input<'_>,
        Output<'_>,
        ROW,
        COL,
        NUM_LAYER,
    >(
        driver,
        input_pins,
        output_pins,
        Some(f),
        crate::keymap::KEYMAP,
        keyboard_config,
    )
    .await;
}
/*
static mut ADC1_DMA_BUF: [u16; 5] = [0u16; 5];
    let p = embassy_stm32::init(Default::default());
    spawner.must_spawn(adc_task(p));

#[embassy_executor::task]
async fn adc_task(mut p: Peripherals) {
    let adc_data: &mut [u16; 10] = singleton!(ADCDAT : [u16; 10] = [0u16; 10]).unwrap();

    let adc = Adc::new(p.ADC1);

    let mut adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(p.DMA2_CH4, adc_data);

    adc.set_sample_sequence(Sequence::One, &mut p.PA0, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Two, &mut p.PA2, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Three, &mut p.PA3, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Four, &mut p.PA4, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Five, &mut p.PA5, SampleTime::CYCLES112);

    // Note that overrun is a big consideration in this implementation. Whatever task is running the adc.read() calls absolutely must circle back around
    // to the adc.read() call before the DMA buffer is wrapped around > 1 time. At this point, the overrun is so significant that the context of
    // what channel is at what index is lost. The buffer must be cleared and reset. This *is* handled here, but allowing this to happen will cause
    // a reduction of performance as each time the buffer is reset, the adc & dma buffer must be restarted.

    // An interrupt executor with a higher priority than other tasks may be a good approach here, allowing this task to wake and read the buffer most
    // frequently.
    let _ = adc.start();
    loop {
        unsafe {
            match adc.read(&mut ADC1_DMA_BUF).await {
                Ok(_data) => {}
                Err(e) => {
                    warn!("Error: {:?}", e);
                    ADC1_DMA_BUF = [0u16; 5];
                    let _ = adc.start();
                }
            }
        }
    }
}
*/
