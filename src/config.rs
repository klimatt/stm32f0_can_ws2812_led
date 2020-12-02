use stm32f0xx_hal::{
    gpio::gpiob::{PB3, PB0, PB1, PB6, PB7, PB4, PB5},
    gpio::gpioa::{PA2, PA5, PA0, PA1, PA7, PA8, PA4, PA11, PA12},
    gpio::gpiof::{PF0, PF1},
    gpio::{Output, PushPull, Analog, PullUp, Alternate, AF2, AF4, AF1},
};
use stm32f0xx_hal::gpio::{Input, PullDown};

pub type CAN_TX_PIN = PA12<Alternate<AF4>>;
pub type CAN_RX_PIN = PA11<Alternate<AF4>>;

pub enum UAVCAN_PRIORITY {
    UcpExceptional = 0,
    UcpImmediate = 1,
    UcpFast = 2,
    UcpHigh = 3,
    UcpNominal = 4,
    UcpLow = 5,
    UCP_Slow = 6,
    UcpOptional = 7
}

pub fn get_uavcan_id(port: u32, node_id: u32, priority: UAVCAN_PRIORITY) -> u32{
    let prio = priority as u32;
    if prio < 7 && port < 32767 && node_id < 127 {
        prio << 26 | port << 8 | node_id
    }
    else{
        0
    }

}