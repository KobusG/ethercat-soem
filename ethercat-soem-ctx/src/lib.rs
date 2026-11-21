//! SOEM low level context

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use ethercat_soem_sys as sys;
use std::{
    ffi::{CStr, CString},
    mem::{self, zeroed},
    os::raw::c_void,
    time::Duration,
};

mod error;
mod group;
mod od_list;
mod oe_list;
mod slave;
mod sm;

pub use crate::{error::*, group::*, od_list::*, oe_list::*, slave::*};

const EC_MAX_GROUP: usize = 2;
const EC_MAX_SLAVE: usize = 200;

/// Size of EEPROM bitmap cache
const EC_MAX_EEP_BITMAP: usize = 128;

/// Size of EEPROM cache buffer
const EC_MAX_EEP_BUF: usize = EC_MAX_EEP_BITMAP << 5;

/// SOEM `ecx_context` wrapper
#[allow(missing_debug_implementations)]
pub struct Ctx {
    #[allow(dead_code)]
    port: Box<sys::ecx_portt>,
    slave_list: Box<[Slave; EC_MAX_SLAVE]>,
    slave_count: Box<i32>,
    group_list: Box<[Group; EC_MAX_GROUP]>,
    #[allow(dead_code)]
    esi_buf: Box<[u8; EC_MAX_EEP_BUF]>,
    #[allow(dead_code)]
    esi_map: Box<[u32; EC_MAX_EEP_BITMAP]>,
    #[allow(dead_code)]
    e_list: Box<[u32; 456]>,
    #[allow(dead_code)]
    idx_stack: Box<[u64; 27]>,
    #[allow(dead_code)]
    ecat_error: Box<u8>,
    dc_time: Box<i64>,
    #[allow(dead_code)]
    sm_comm_type: Box<[u8; 10]>,
    #[allow(dead_code)]
    pdo_assign: Box<[u8; 514]>,
    #[allow(dead_code)]
    pdo_desc: Box<[u8; 1026]>,
    #[allow(dead_code)]
    eep_sm: Box<[u16; 6]>,
    #[allow(dead_code)]
    eep_fmmu: Box<[u16; 4]>,

    /// The original context
    ecx_ctx: sys::ecx_context,

    /// Last FoE DATA payload.
    foe_last: Box<[u8; 1487]>,
    /// Length of `foe_last`.
    foe_last_len: usize,

    /// I/O map
    pub io_map: [u8; 4096],
}

impl Default for Ctx {
    fn default() -> Self {
        let mut port = Box::new(sys::ecx_portt {
            _bindgen_opaque_blob: [0; 6502],
        });
        let mut slave_list: Box<[Slave; EC_MAX_SLAVE]> = Box::new(unsafe { zeroed() });
        let mut slave_count = Box::new(0);
        let mut group_list: Box<[Group; EC_MAX_GROUP]> = Box::new(unsafe { zeroed() });
        let mut esi_buf: Box<[u8; EC_MAX_EEP_BUF]> = Box::new([0; EC_MAX_EEP_BUF]);
        let mut esi_map: Box<[u32; EC_MAX_EEP_BITMAP]> = Box::new([0; EC_MAX_EEP_BITMAP]);
        let mut e_list: Box<[u32; 456]> = Box::new([0; 456]);
        let mut idx_stack: Box<[u64; 27]> = Box::new([0; 27]);
        let mut ecat_error: Box<u8> = Box::new(0);
        let mut dc_time = Box::new(0);
        let mut sm_comm_type: Box<[u8; 10]> = Box::new([0; 10]);
        let mut pdo_assign: Box<[u8; 514]> = Box::new([0; 514]);
        let mut pdo_desc: Box<[u8; 1026]> = Box::new([0; 1026]);
        let mut eep_sm: Box<[u16; 6]> = Box::new([0; 6]);
        let mut eep_fmmu: Box<[u16; 4]> = Box::new([0; 4]);
        let foe_last: Box<[u8; 1487]> = Box::new([0; 1487]);
        let foe_last_len = 0;
        let io_map = [0; 4096];

        // The original context
        let ecx_ctx = sys::ecx_context {
            port: &mut *port,
            slavelist: slave_list.as_mut_ptr() as *mut sys::ec_slave,
            slavecount: &mut *slave_count,
            maxslave: EC_MAX_SLAVE as i32,
            grouplist: group_list.as_mut_ptr() as *mut sys::ec_group,
            maxgroup: EC_MAX_GROUP as i32,
            esibuf: esi_buf.as_mut_ptr(),
            esimap: esi_map.as_mut_ptr(),
            esislave: 0,
            elist: &mut *e_list,
            idxstack: &mut *idx_stack,
            ecaterror: &mut *ecat_error,
            DCtime: &mut *dc_time,
            SMcommtype: &mut *sm_comm_type,
            PDOassign: &mut *pdo_assign,
            PDOdesc: &mut *pdo_desc,
            eepSM: &mut *eep_sm,
            eepFMMU: &mut *eep_fmmu,
            FOEhook: None,
            EOEhook: None,
            manualstatechange: 0,
            userdata: std::ptr::null_mut(),
        };

        Self {
            port,
            slave_list,
            slave_count,
            group_list,
            esi_buf,
            esi_map,
            e_list,
            idx_stack,
            ecat_error,
            dc_time,
            sm_comm_type,
            pdo_assign,
            pdo_desc,
            eep_sm,
            eep_fmmu,
            ecx_ctx,
            foe_last,
            foe_last_len,
            io_map,
        }
    }
}

impl Ctx {
    /// Initialise lib in single NIC mode.
    ///
    /// Return > 0 if OK.
    pub fn init(&mut self, iface: CString) -> i32 {
        unsafe { sys::ecx_init(&mut self.ecx_ctx, iface.as_ptr()) }
    }
    pub fn config_init(&mut self, use_table: bool) -> i32 {
        unsafe { sys::ecx_config_init(&mut self.ecx_ctx, if use_table { 1 } else { 0 }) }
    }
    pub fn config_map_group(&mut self, group: u8) -> i32 {
        unsafe {
            sys::ecx_config_map_group(
                &mut self.ecx_ctx,
                self.io_map.as_mut_ptr() as *mut std::ffi::c_void,
                group,
            )
        }
    }
    pub fn config_dc(&mut self) -> u8 {
        unsafe { sys::ecx_configdc(&mut self.ecx_ctx) }
    }
    pub const fn slave_count(&self) -> usize {
        *self.slave_count as usize
    }
    pub fn slaves(&self) -> &[Slave; EC_MAX_SLAVE] {
        &self.slave_list
    }
    pub fn slaves_mut(&mut self) -> &mut [Slave; EC_MAX_SLAVE] {
        &mut self.slave_list
    }
    pub const fn groups(&self) -> &[Group; EC_MAX_GROUP] {
        &self.group_list
    }
    /// Write slave state, if slave = 0 then write to all slaves.
    ///
    /// The function does not check if the actual state is changed.
    /// It returns Workcounter or `EC_NOFRAME` (= `-1`),
    pub fn write_state(&mut self, slave: u16) -> i32 {
        unsafe { sys::ecx_writestate(&mut self.ecx_ctx, slave) }
    }
    /// Check actual slave state.
    ///
    /// This is a blocking function.
    /// To refresh the state of all slaves `read_state()` should be called.
    ///
    /// Parameter `slave` = Slave number, 0 = all slaves (only the "slavelist[0].state" is refreshed).
    ///
    /// It returns requested state, or found state after timeout.
    pub fn state_check(&mut self, slave: u16, state: u16, timeout: Duration) -> u16 {
        unsafe { sys::ecx_statecheck(&mut self.ecx_ctx, slave, state, timeout.as_micros() as i32) }
    }
    /// Read all slave states in ec_slave.
    ///
    /// It returns the lowest state found,
    pub fn read_state(&mut self) -> i32 {
        unsafe { sys::ecx_readstate(&mut self.ecx_ctx) }
    }
    pub fn send_processdata(&mut self) -> i32 {
        unsafe { sys::ecx_send_processdata(&mut self.ecx_ctx) }
    }
    pub fn receive_processdata(&mut self, timeout: Duration) -> i32 {
        unsafe { sys::ecx_receive_processdata(&mut self.ecx_ctx, timeout.as_micros() as i32) }
    }
    pub fn read_od_list(&mut self, slave: u16, od_list: &mut OdList) -> i32 {
        unsafe { sys::ecx_readODlist(&mut self.ecx_ctx, slave, &mut od_list.0) }
    }
    pub fn read_od_description(&mut self, item: u16, od_list: &mut OdList) -> i32 {
        unsafe { sys::ecx_readODdescription(&mut self.ecx_ctx, item, &mut od_list.0) }
    }
    pub fn read_oe(&mut self, item: u16, od_list: &mut OdList, oe_list: &mut OeList) -> i32 {
        unsafe { sys::ecx_readOE(&mut self.ecx_ctx, item, &mut od_list.0, &mut oe_list.0) }
    }
    pub fn sdo_read<'t>(
        &mut self,
        slave: u16,
        idx: u16,
        sub_idx: u8,
        access_complete: bool,
        target: &'t mut [u8],
        timeout: Duration,
    ) -> (i32, &'t mut [u8]) {
        let mut size = mem::size_of_val(target) as i32;
        let timeout = timeout.as_micros() as i32; //TODO: check overflow
        let wkc = unsafe {
            sys::ecx_SDOread(
                &mut self.ecx_ctx,
                slave,
                idx,
                sub_idx,
                if access_complete { 1 } else { 0 },
                &mut size,
                target.as_mut_ptr() as *mut c_void,
                timeout,
            )
        };
        if wkc <= 0 {
            (wkc, target)
        } else {
            (wkc, &mut target[..size as usize])
        }
    }
    pub fn sdo_write(
        &mut self,
        slave: u16,
        idx: u16,
        sub_idx: u8,
        access_complete: bool,
        data: &[u8],
        timeout: Duration,
    ) -> i32 {
        let size = mem::size_of_val(data) as i32;
        let timeout = timeout.as_micros() as i32; //TODO: check overflow
        unsafe {
            sys::ecx_SDOwrite(
                &mut self.ecx_ctx,
                slave,
                idx,
                sub_idx,
                if access_complete { 1 } else { 0 },
                size,
                data.as_ptr() as *mut c_void,
                timeout,
            )
        }
    }
    /// Read a FoE file using acknowledged DATA packets.
    ///
    /// Returns the file length, or the FoE error code reported by the slave.
    pub fn foe_read(
        &mut self,
        slave: u16,
        filename: &CString,
        password: u32,
        target: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, u32> {
        let timeout = timeout.as_micros().min(i32::MAX as u128) as i32;
        self.foe_flush(slave);
        let max_data = self.foe_send(slave, 1, password, filename.to_bytes(), timeout)? - 12;
        let mut written = 0;
        let mut expected = 1u32;
        loop {
            let (opcode, value, data) = self.foe_receive(slave, timeout)?;
            if opcode == 5 {
                return Err(value);
            }
            if opcode != 3 || value != expected {
                return Err(u32::MAX);
            }
            let end = written + data;
            if end > target.len() {
                return Err(u32::MAX);
            }
            self.foe_copy_data(&mut target[written..end])?;
            written = end;
            self.foe_send(slave, 4, expected, &[], timeout)?;
            if data < max_data {
                return Ok(written);
            }
            expected = expected.wrapping_add(1);
        }
    }

    /// Write a FoE file using acknowledged DATA packets.
    pub fn foe_write(
        &mut self,
        slave: u16,
        filename: &CString,
        password: u32,
        data: &[u8],
        timeout: Duration,
    ) -> Result<(), u32> {
        let timeout = timeout.as_micros().min(i32::MAX as u128) as i32;
        self.foe_flush(slave);
        let max_data = self.foe_send(slave, 2, password, filename.to_bytes(), timeout)? - 12;
        let (opcode, value, _) = self.foe_receive(slave, timeout)?;
        if opcode == 5 {
            return Err(value);
        }
        if opcode != 4 || value != 0 {
            return Err(u32::MAX);
        }
        let mut offset = 0;
        let mut packet = 1u32;
        loop {
            let end = (offset + max_data).min(data.len());
            self.foe_send(slave, 3, packet, &data[offset..end], timeout)?;
            let (opcode, value, _) = self.foe_receive(slave, timeout)?;
            if opcode == 5 {
                return Err(value);
            }
            if opcode != 4 || value != packet {
                return Err(u32::MAX);
            }
            if end - offset < max_data {
                return Ok(());
            }
            offset = end;
            packet = packet.wrapping_add(1);
        }
    }

    fn foe_flush(&mut self, slave: u16) {
        let mut mailbox: sys::ec_mbxbuft = unsafe { mem::zeroed() };
        unsafe {
            sys::ecx_mbxreceive(&mut self.ecx_ctx, slave, &mut mailbox, 0);
        }
    }

    fn foe_mailbox(&self, slave: u16) -> Result<(usize, usize), u32> {
        let index = usize::from(slave).checked_sub(1).ok_or(u32::MAX)?;
        let device = self.slaves().get(index).ok_or(u32::MAX)?;
        if device.mbx_proto() & 0x0008 == 0 || device.mbx_l() < 12 || device.mbx_rl() < 12 {
            return Err(u32::MAX);
        }
        Ok((usize::from(device.mbx_l()), usize::from(device.mbx_rl())))
    }

    fn foe_send(
        &mut self,
        slave: u16,
        opcode: u16,
        value: u32,
        data: &[u8],
        timeout: i32,
    ) -> Result<usize, u32> {
        let (mailbox_len, _) = self.foe_mailbox(slave)?;
        let service_len = 6usize.checked_add(data.len()).ok_or(u32::MAX)?;
        let frame_len = 6usize.checked_add(service_len).ok_or(u32::MAX)?;
        if frame_len > mailbox_len || frame_len > 1487 {
            return Err(u32::MAX);
        }
        let index = usize::from(slave).checked_sub(1).ok_or(u32::MAX)?;
        let counter = (self.slaves()[index].mbx_cnt() % 7) + 1;
        self.slaves_mut()[index].set_mbx_cnt(counter);

        let mut mailbox: sys::ec_mbxbuft = unsafe { mem::zeroed() };
        mailbox[..2].copy_from_slice(&(service_len as u16).to_le_bytes());
        mailbox[5] = 0x04 | (counter << 4);
        mailbox[6..8].copy_from_slice(&opcode.to_le_bytes());
        mailbox[8..12].copy_from_slice(&value.to_le_bytes());
        mailbox[12..frame_len].copy_from_slice(data);
        let sent = unsafe { sys::ecx_mbxsend(&mut self.ecx_ctx, slave, &mut mailbox, timeout) };
        if sent <= 0 {
            return Err(u32::MAX);
        }
        Ok(mailbox_len)
    }

    fn foe_receive(&mut self, slave: u16, timeout: i32) -> Result<(u16, u32, usize), u32> {
        let (_, read_len) = self.foe_mailbox(slave)?;
        let mut mailbox: sys::ec_mbxbuft = unsafe { mem::zeroed() };
        let received = unsafe { sys::ecx_mbxreceive(&mut self.ecx_ctx, slave, &mut mailbox, timeout) };
        if received <= 0 {
            return Err(u32::MAX);
        }
        let service_len = usize::from(u16::from_le_bytes([mailbox[0], mailbox[1]]));
        if mailbox[5] & 0x0f != 0x04 || service_len < 6 || service_len + 6 > read_len || service_len + 6 > 1487 {
            return Err(u32::MAX);
        }
        let opcode = u16::from_le_bytes([mailbox[6], mailbox[7]]);
        let value = u32::from_le_bytes([mailbox[8], mailbox[9], mailbox[10], mailbox[11]]);
        self.foe_last_len = service_len - 6;
        self.foe_last[..self.foe_last_len].copy_from_slice(&mailbox[12..12 + self.foe_last_len]);
        Ok((opcode, value, self.foe_last_len))
    }

    fn foe_copy_data(&self, output: &mut [u8]) -> Result<(), u32> {
        if output.len() != self.foe_last_len {
            return Err(u32::MAX);
        }
        output.copy_from_slice(&self.foe_last[..self.foe_last_len]);
        Ok(())
    }

    pub fn voe_exchange(
        &mut self,
        slave: u16,
        vendor_id: u32,
        vendor_type: u16,
        payload: &[u8],
        response: &mut [u8],
        timeout: Duration,
    ) -> Result<(u16, usize), ()> {
        let index = usize::from(slave).checked_sub(1).ok_or(())?;
        let device = self.slaves_mut().get_mut(index).ok_or(())?;
        if device.mbx_proto() & 0x0020 == 0 || device.mbx_l() < 12 || device.mbx_rl() < 12 {
            return Err(());
        }
        let body_len = 6usize.checked_add(payload.len()).ok_or(())?;
        let frame_len = 6usize.checked_add(body_len).ok_or(())?;
        if frame_len > device.mbx_l() as usize || frame_len > 1487 {
            return Err(());
        }
        let counter = (device.mbx_cnt() % 7) + 1;
        device.set_mbx_cnt(counter);
        let read_mailbox_len = device.mbx_rl() as usize;

        let mut mailbox: sys::ec_mbxbuft = unsafe { mem::zeroed() };
        mailbox[0..2].copy_from_slice(&(body_len as u16).to_le_bytes());
        mailbox[5] = 0x0f | (counter << 4);
        mailbox[6..10].copy_from_slice(&vendor_id.to_le_bytes());
        mailbox[10..12].copy_from_slice(&vendor_type.to_le_bytes());
        mailbox[12..frame_len].copy_from_slice(payload);
        let timeout = timeout.as_micros().min(i32::MAX as u128) as i32;
        let sent = unsafe { sys::ecx_mbxsend(&mut self.ecx_ctx, slave, &mut mailbox, timeout) };
        if sent <= 0 {
            return Err(());
        }
        let received =
            unsafe { sys::ecx_mbxreceive(&mut self.ecx_ctx, slave, &mut mailbox, timeout) };
        if received <= 0 {
            return Err(());
        }
        let response_len = u16::from_le_bytes([mailbox[0], mailbox[1]]) as usize;
        if mailbox[5] & 0x0f != 0x0f
            || response_len < 6
            || response_len > response.len() + 6
            || response_len + 6 > read_mailbox_len
            || response_len + 6 > 1487
        {
            return Err(());
        }
        let reply_vendor = u32::from_le_bytes(mailbox[6..10].try_into().map_err(|_| ())?);
        if reply_vendor != vendor_id {
            return Err(());
        }
        let message_type = u16::from_le_bytes([mailbox[10], mailbox[11]]);
        let payload_len = response_len - 6;
        response[..payload_len].copy_from_slice(&mailbox[12..12 + payload_len]);
        Ok((message_type, payload_len))
    }
    pub const fn max_group(&self) -> i32 {
        self.ecx_ctx.maxgroup
    }
    pub fn is_err(&mut self) -> bool {
        unsafe { sys::ecx_iserror(&mut self.ecx_ctx) != 0 }
    }
    pub fn pop_error(&mut self) -> Option<Error> {
        let mut ec: sys::ec_errort = unsafe { zeroed() };
        if unsafe { sys::ecx_poperror(&mut self.ecx_ctx, &mut ec) } != 0 {
            Some(Error::from(ec))
        } else {
            None
        }
    }
    pub const fn dc_time(&self) -> i64 {
        *self.dc_time
    }
}

fn c_array_to_string(data: *const i8) -> String {
    unsafe { CStr::from_ptr(data).to_string_lossy().into_owned() }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn context_wrapper() {
        let mut wrapper = Ctx::default();
        wrapper.port._bindgen_opaque_blob[3] = 5;
        assert_eq!(
            unsafe { (*wrapper.ecx_ctx.port)._bindgen_opaque_blob[3] },
            5
        );
        assert_eq!(wrapper.slave_list.len(), 200);
        assert_eq!(wrapper.slave_list[7].0.ALstatuscode, 0);
        wrapper.slave_list[7].0.ALstatuscode = 33;
        assert_eq!(
            unsafe {
                std::slice::from_raw_parts_mut(wrapper.ecx_ctx.slavelist, 200)[7].ALstatuscode
            },
            33
        );
    }
}
