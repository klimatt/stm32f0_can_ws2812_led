use crate::config::{CAN_TX_PIN, CAN_RX_PIN};

use stm32f0xx_hal::{
    stm32::{CAN},
    prelude::*,
};
use cortex_m::asm::delay;
use rtt_target::rprintln;

pub enum CanMode{
    NormalMode = 0,
    SilentMode = 1,
    LoopBackMode = 2,
    LoopBackSilentMode = 3
}

pub enum IdType{
    Standard = 0,
    Extended = 1
}

pub enum AutomaticRetransmission{
    Enabled = 0,
    Disabled = 1
}

pub enum AutomaticBussOffManagement{
    Enabled = 0,
    Disabled = 1
}

pub enum AutomaticWakeUpMode{
    Enabled = 0,
    Disabled = 1
}

pub enum ErrorState{
    BussOff = 0,
    ErrorPassive = 1,
    Warning = 2,
    NoError = 3
}

pub enum BitRate{
    _1Mbs = 0,
    _500Kbs = 1,
    _100Kbs = 2
}

pub struct Error{

}

pub struct CanParams{
    pub(crate) work_mode: CanMode,
    pub(crate) automatic_retransmission: AutomaticRetransmission,
    pub(crate) automatic_busoff_management : AutomaticBussOffManagement,
    pub(crate) auto_wake_up : AutomaticWakeUpMode,
    pub(crate) bitrate: BitRate
}

pub struct Can{
    tx_pin : CAN_TX_PIN,
    rx_pin : CAN_RX_PIN,
    can_reg : CAN,
    can_params: CanParams,
    available_tx_mailbox: [bool; 3]
}

impl Can{
    pub fn new(tx_pin: CAN_TX_PIN, rx_pin: CAN_RX_PIN, can_reg: CAN, can_params: CanParams)->Can{
        let mut cfg_can_timeout = 1_000_000_u32;

        can_reg.mcr.modify(|_,w| w.inrq().set_bit());
        while can_reg.msr.read().inak().bit_is_clear() {
            cfg_can_timeout = cfg_can_timeout - 1;
            if cfg_can_timeout == 0 {
                rprintln!("CAN: inrq enable fail\n");
                break;
            }
        }
        can_reg.mcr.modify(|_, w| w.sleep().clear_bit());
        cfg_can_timeout = 1_000_000_u32;
        while can_reg.msr.read().slak().bit_is_set() {
            cfg_can_timeout = cfg_can_timeout - 1;
            if cfg_can_timeout == 0 {
                rprintln!("CAN: sleep fail\n");
                break;
            }
        }

        match can_params.automatic_busoff_management{
            AutomaticBussOffManagement::Enabled => can_reg.mcr.modify(|_, w| w.abom().set_bit()),
            AutomaticBussOffManagement::Disabled => can_reg.mcr.modify(|_, w| w.abom().clear_bit())
        }

        match can_params.automatic_retransmission{
            AutomaticRetransmission::Enabled => can_reg.mcr.modify(|_, w| w.nart().clear_bit()),
            AutomaticRetransmission::Disabled => can_reg.mcr.modify(|_, w| w.nart().set_bit())
        }

        match can_params.auto_wake_up{
            AutomaticWakeUpMode::Enabled => can_reg.mcr.modify(|_, w| w.awum().set_bit()),
            AutomaticWakeUpMode::Disabled => can_reg.mcr.modify(|_, w| w.awum().clear_bit())
        }

        match can_params.work_mode{
            CanMode::LoopBackMode => can_reg.btr.modify(|_,w| w.lbkm().enabled().silm().normal()),
            CanMode::LoopBackSilentMode => can_reg.btr.modify(|_,w| w.lbkm().enabled().silm().silent()),
            CanMode::SilentMode => can_reg.btr.modify(|_,w| w.lbkm().disabled().silm().silent()),
            CanMode::NormalMode => can_reg.btr.modify(|_,w| w.lbkm().disabled().silm().normal())
        }
        match can_params.bitrate{
            BitRate::_1Mbs => can_reg.btr.modify(|_,w| unsafe{w.sjw().bits(1).ts1().bits(2).ts2().bits(3).brp().bits(0)}),
            BitRate::_500Kbs => can_reg.btr.modify(|_,w| unsafe{w.sjw().bits(3).ts1().bits(4).ts2().bits(1).brp().bits(0)}),
            BitRate::_100Kbs => can_reg.btr.modify(|_,w| unsafe{w.sjw().bits(4).ts1().bits(3).ts2().bits(1).brp().bits(0)}),
        }

        can_reg.ier.modify(|_,w|w.errie().disabled());
        can_reg.ier.modify(|_,w|w.bofie().disabled());
        can_reg.ier.modify(|_,w|w.epvie().disabled());
        can_reg.ier.modify(|_,w|w.ewgie().disabled());
        can_reg.ier.modify(|_,w|w.lecie().disabled());
        can_reg.ier.modify(|_,w|w.fmpie0().enabled());
        can_reg.ier.modify(|_,w|w.fmpie1().enabled());
        can_reg.ier.modify(|_,w|w.tmeie().enabled());

        can_reg.mcr.modify(|_,w| w.inrq().clear_bit());
        cfg_can_timeout = 1_000_000_u32;
        while can_reg.msr.read().inak().bit_is_set() {
            cfg_can_timeout = cfg_can_timeout - 1;
            if cfg_can_timeout == 0 {
                rprintln!("CAN: inrq dis fail\n");
                break;
            }
        }

        // filters need to fix\\
        can_reg.fmr.modify(|_,w|w.finit().set_bit());
        can_reg.fa1r.modify(|_,w|w.fact0().clear_bit());

        can_reg.fb[0].fr1.modify(|_,w|unsafe{w.bits(0)});
        can_reg.fb[0].fr2.modify(|_,w|unsafe{w.bits(0)});

        can_reg.fa1r.modify(|_,w|w.fact0().set_bit());

        can_reg.fmr.modify(|_,w|w.finit().clear_bit());


        ////////////////////////////////////////

        Can{
            tx_pin,
            rx_pin,
            can_reg,
            can_params,
            available_tx_mailbox:[true, true, true]
        }
    }

    pub fn reboot(&mut self){
        let mut cfg_can_timeout = 1_000_000_u32;
        self.can_reg.mcr.modify(|_,w| w.inrq().set_bit());
        while self.can_reg.msr.read().inak().bit_is_clear() {
            cfg_can_timeout = cfg_can_timeout - 1;
            if cfg_can_timeout == 0 {
                rprintln!("CAN: inrq enable fail\n");
                break;
            }
        }
        self.can_reg.mcr.modify(|_, w| w.sleep().clear_bit());
        cfg_can_timeout = 1_000_000_u32;
        while self.can_reg.msr.read().slak().bit_is_set() {
            cfg_can_timeout = cfg_can_timeout - 1;
            if cfg_can_timeout == 0 {
                rprintln!("CAN: sleep fail\n");
                break;
            }
        }
        match self.can_params.work_mode{
            CanMode::LoopBackMode => self.can_reg.btr.modify(|_,w| w.lbkm().enabled().silm().normal()),
            CanMode::LoopBackSilentMode => self.can_reg.btr.modify(|_,w| w.lbkm().enabled().silm().silent()),
            CanMode::SilentMode => self.can_reg.btr.modify(|_,w| w.lbkm().disabled().silm().silent()),
            CanMode::NormalMode => self.can_reg.btr.modify(|_,w| w.lbkm().disabled().silm().normal())
        }

        self.can_reg.mcr.modify(|_,w| w.inrq().clear_bit());
    }


    pub fn write_to_mailbox(&mut self, id_type: IdType, transmit_id: u32, data: &[u8]){
        for i in 0..self.available_tx_mailbox.len() {
            match self.available_tx_mailbox[i]{
                false => continue,
                true => {
                    self.can_reg.tx[i].tir.modify(|_, w| w.rtr().data());
                    match id_type {
                        IdType::Standard => {
                            self.can_reg.tx[i].tir.modify(|_, w| w.ide().standard());
                            self.can_reg.tx[i].tir.modify(|_, w| unsafe { w.stid().bits(transmit_id as u16) });
                        },
                        IdType::Extended => {
                            self.can_reg.tx[i].tir.modify(|_, w| w.ide().extended());
                            self.can_reg.tx[i].tir.modify(|_, w| unsafe { w.stid().bits((transmit_id >> 18) as u16) });
                            self.can_reg.tx[i].tir.modify(|_, w| unsafe { w.exid().bits(transmit_id) });
                        }
                    }
                    let dlc = data.len() as u8;
                    self.can_reg.tx[i].tdtr.write(|w| unsafe { w.dlc().bits(dlc) });
                    if dlc > 4 {
                        let data = data.as_ptr() as *const _ as *const u64;
                        self.can_reg.tx[i].tdhr.write(|w| unsafe { w.bits((*data >> 32) as u32) });
                        self.can_reg.tx[i].tdlr.write(|w| unsafe { w.bits(*data as u32) });
                    } else if dlc > 0 {
                        let data = data.as_ptr() as *const _ as *const u32;
                        self.can_reg.tx[i].tdlr.write(|w| unsafe { w.bits(*data as u32) });
                    }
                    self.can_reg.tx[i].tir.modify(|_, w| w.txrq().set_bit());
                    self.available_tx_mailbox[i] = false;
                    break;
                }
            }

        }
    }

    pub fn irq_state_machine<F: FnMut(u32, &[u8])>(&mut self, mut f: F) {
        let fifo_rx_pending: [u8; 2] = [self.can_reg.rfr[0].read().fmp().bits(), self.can_reg.rfr[1].read().fmp().bits()];
        let tx_state = self.can_reg.tsr.read();
        let tx_err_state = self.can_reg.esr.read();
        let master_err_state = self.can_reg.msr.read();


        if tx_err_state.epvf().bit_is_set() == true{
            rprintln!("CAN: Error passive");
            rprintln!("TEC: {}", tx_err_state.tec().bits());
            rprintln!("REC: {}", tx_err_state.rec().bits());
            self.can_reg.msr.modify(|_,w| w.erri().clear_bit())
        }

        if tx_err_state.ewgf().bit_is_set() == true{
            rprintln!("CAN: Warning");
            self.can_reg.msr.modify(|_,w| w.erri().clear_bit());
            self.can_reg.tsr.write(|w|w.abrq0().set_bit());
            self.can_reg.tsr.write(|w|w.abrq1().set_bit());
            self.can_reg.tsr.write(|w|w.abrq2().set_bit());
            self.available_tx_mailbox[0] = true;
            self.available_tx_mailbox[1] = true;
            self.available_tx_mailbox[2] = true;
        }

        if tx_err_state.boff().bit_is_set() == true{
            rprintln!("CAN: busOFF");
            self.can_reg.msr.modify(|_,w| w.erri().clear_bit());
        }
        if tx_state.rqcp0().bit_is_set() == true{
            self.available_tx_mailbox[0] = true;
            self.can_reg.tsr.write(|w|w.rqcp0().set_bit())
        }
        if tx_state.rqcp1().bit_is_set() == true{
            self.available_tx_mailbox[1] = true;
            self.can_reg.tsr.write(|w|w.rqcp1().set_bit())
        }
        if tx_state.rqcp2().bit_is_set() == true{
            self.available_tx_mailbox[2] = true;
            self.can_reg.tsr.write(|w|w.rqcp2().set_bit())
        }
        for i in 0..=1{
            if fifo_rx_pending[i] != 0b00{
                let mut rx_id: u32 = 0;
                match self.can_reg.rx[i].rir.read().ide().bits()
                {
                    true => {rx_id = self.can_reg.rx[i].rir.read().exid().bits();},
                    false => {rx_id = self.can_reg.rx[i].rir.read().stid().bits() as u32;}
                }
                let rx_dlc = self.can_reg.rx[i].rdtr.read().dlc().bits();
                let data_raw: u64 = (self.can_reg.rx[i].rdhr.read().bits() as u64) << 32 | (self.can_reg.rx[i].rdlr.read().bits()) as u64;
                let data = &data_raw as *const _ as * const u8;
                let data = unsafe{core::slice::from_raw_parts(data, rx_dlc as usize)};
                self.can_reg.rfr[i].modify(|_, w| w.rfom().release());
                f(rx_id, data);
            }
        }
    }
}