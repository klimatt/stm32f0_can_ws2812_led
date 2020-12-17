use stm32f0xx_hal::{
    gpio::gpioa::{PA5, PA6, PA7, PA11, PA12},
    gpio::gpiob::{PB3, PB4, PB5},
    gpio::{Alternate, AF4, AF0},
    spi::Spi,
    stm32::{SPI1}
};

pub type CAN_TX_PIN = PA12<Alternate<AF4>>;
pub type CAN_RX_PIN = PA11<Alternate<AF4>>;
pub type SCK_PIN = PB3<Alternate<AF0>>;
pub type MISO_PIN = PB4<Alternate<AF0>>;
pub type MOSI_PIN = PB5<Alternate<AF0>>;

pub type SPI_TYPE = Spi<SPI1, SCK_PIN, MISO_PIN, MOSI_PIN>;


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

pub fn update_reg_by_bit_pos(src: u32, pos: u32, val: u32) -> u32{
    let tmp = (src >> pos) | val;
    ((tmp << pos) + src)
}