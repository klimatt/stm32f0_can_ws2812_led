#![no_std]
#![no_main]

// pick a panicking behavior
//use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

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
};
#[app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        can: can::Can,
    }
    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        let mut dp: stm32::Peripherals = ctx.device;
        let cs = unsafe {CriticalSection::new()};
        let rcc = dp.RCC;

        rcc.apb1enr.modify(|_, w| w.canen().enabled()); // can time enb

        let mut clock = rcc
            .configure()
            .sysclk(8.mhz())
            .freeze(&mut dp.FLASH);

        let gpioa = dp.GPIOA.split(&mut clock);
        let can_rx: config::CAN_RX_PIN = gpioa.pa11.into_alternate_af4(&cs);
        let can_tx: config::CAN_TX_PIN = gpioa.pa12.into_alternate_af4(&cs);
        let can_params: can::CanParams = can::CanParams{
            work_mode: can::CanMode::NormalMode,
            automatic_retransmission: can::AutomaticRetransmission::Enabled,
            automatic_busoff_management: can::AutomaticBussOffManagement::Enabled,
            auto_wake_up: can::AutomaticWakeUpMode::Enabled,
            bitrate: can::BitRate::_1Mbs
        };

        let can  = can::Can::new(
            can_tx,
            can_rx,
            dp.CAN,
            can_params
        );
        init::LateResources {
            can
        }
    }
    #[idle(resources = [can])]
    fn idle(ctx: idle::Context) -> ! {
        let mut can = ctx.resources.can;
        loop {
            delay(1_000_00);
            can.lock(|can| {
                can.write_to_mailbox(can::IdType::Extended, 0x00000005, &[5, 6, 1 , 1, 1]);
                can.write_to_mailbox(can::IdType::Extended, 0x00000003, &[3, 4, 1 , 1, 1]);
                can.write_to_mailbox(can::IdType::Extended, 0x00000001, &[1, 2, 1 , 1, 1]);
            });
            rprintln!("*************************");
            delay(1_000_00);
        }

    }

    #[task(binds = CEC_CAN, priority = 4 , resources = [can])]
    fn can_irq(ctx: can_irq::Context){
        //rprintln!("CAN_IRQ\n");
        let can: &mut can::Can =  ctx.resources.can;
        can.irq_state_machine(|id, data|{
            rprintln!("CAN_IRQ: id: {:x}; Data: {:?}", id, data);

        });
    }
};


use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use nb::Error;
use core::borrow::BorrowMut;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("Panic: {:?}", info);
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}