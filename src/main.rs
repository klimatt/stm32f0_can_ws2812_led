#![no_std]
#![no_main]

mod config;
mod can;

use rtic::app;
use cortex_m::asm::delay;
use rtt_target::{rprintln, rtt_init_print};
use cortex_m::interrupt::{free as disable_interrupts, CriticalSection};
use stm32f0xx_hal::time::Hertz;
use stm32f0xx_hal::{
    prelude::*,
    stm32,
    spi::Spi
};

use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_spi::{Ws2812 as ws2812};

#[app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        can: can::Can,
        ws2812: ws2812<config::SPI_TYPE>
    }
    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        let mut dp: stm32::Peripherals = ctx.device;
        let cs = unsafe {CriticalSection::new()};
        let mut rcc = dp.RCC;

        rcc.apb1enr.modify(|_, w| w.canen().enabled()); // can time enb

        let mut clock = rcc
            .configure()
            .sysclk( 48.mhz())
            .freeze(&mut dp.FLASH);

        let gpioa = dp.GPIOA.split(&mut clock);
        let gpiob = dp.GPIOB.split(&mut clock);
        let can_rx: config::CAN_RX_PIN = gpioa.pa11.into_alternate_af4(&cs);
        let can_tx: config::CAN_TX_PIN = gpioa.pa12.into_alternate_af4(&cs);

        let can_params: can::CanParams = can::CanParams{
            work_mode: can::CanMode::NormalMode,
            automatic_retransmission: can::AutomaticRetransmission::Enabled,
            automatic_busoff_management: can::AutomaticBussOffManagement::Enabled,
            auto_wake_up: can::AutomaticWakeUpMode::Enabled,
            pclk_Hz: clock.clocks.pclk(),
            bitrate: can::BitRate::_1Mbs
        };

        let can  = can::Can::new(
            can_tx,
            can_rx,
            dp.CAN,
            can_params
        );

        let sck: config::SCK_PIN = gpiob.pb3.into_alternate_af0(&cs);
        let miso: config::MISO_PIN = gpiob.pb4.into_alternate_af0(&cs);
        let mosi: config::MOSI_PIN = gpiob.pb5.into_alternate_af0(&cs);

        let spi = Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            ws2812_spi::MODE,
            3_000_000.hz(),
            &mut clock,
        );

        let ws2812 = ws2812::new(spi);


        init::LateResources {
            can,
            ws2812
        }
    }
    #[idle(resources = [can])]
    fn idle(ctx: idle::Context) -> ! {
        let mut can = ctx.resources.can;
        loop {
            delay(6_000_000);
            rprintln!("*************************");
            delay(6_000_000);
        }

    }

    #[task(binds = CEC_CAN, priority = 4 , resources = [can, ws2812])]
    fn can_irq(ctx: can_irq::Context){
        let can: &mut can::Can = ctx.resources.can;
        let ws2812: &mut ws2812<SPI_TYPE> = ctx.resources.ws2812;
        can.irq_state_machine(|id, data|{
            rprintln!("CAN_IRQ: id: {:x}; Data: {:?}", id, data);
            let mut color = [RGB8::default(); 16];
            for i in 0..16{
                color[i] = RGB8::new(data[0], data[1], data[2]);
            }
            ws2812.write(brightness(color.iter().cloned(), 100)).unwrap();
        });
        if can.receive_flag {
            can.write_to_mailbox(can::IdType::Extended, 0x00000001, &[]);
        }
    }
};


use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use nb::Error;
use core::borrow::BorrowMut;
use crate::config::SPI_TYPE;
use smart_leds::colors::{CORAL, RED, AQUA};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("Panic: {:?}", info);
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}