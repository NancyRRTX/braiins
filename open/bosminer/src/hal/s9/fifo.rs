/// This module provides thin API to access memory-mapped FPGA registers
/// and associated interrupts.
/// Exports FIFO management/send/receive and register access.
use crate::error::{self, ErrorKind};
use failure::ResultExt;
use std::time::Duration;

use s9_io::hchainio0;

#[cfg(not(feature = "hctl_polling"))]
mod fifo_irq;
#[cfg(feature = "hctl_polling")]
mod fifo_poll;

/// How long to wait for RX interrupt
const FIFO_READ_TIMEOUT: Duration = Duration::from_millis(5);

#[cfg(feature = "hctl_polling")]
pub struct HChainFifo<'a> {
    pub hash_chain_io: &'a hchainio0::RegisterBlock,
}

#[cfg(not(feature = "hctl_polling"))]
pub struct HChainFifo<'a> {
    pub hash_chain_io: &'a hchainio0::RegisterBlock,
    work_tx_irq: uio::UioDevice,
    work_rx_irq: uio::UioDevice,
    cmd_rx_irq: uio::UioDevice,
}

/// Performs memory mapping of IP core's register block
/// * `hashboard_idx` is the number of chain (numbering must match in device-tree)
fn mmap(hashboard_idx: usize) -> error::Result<*const hchainio0::RegisterBlock> {
    let uio_name = format!("chain{}-mem", hashboard_idx - 1);
    let uio = uio::UioDevice::open_by_name(&uio_name)?;
    let mem = uio
        .map_mapping(0)
        .with_context(|_| ErrorKind::UioDevice(uio_name, "cannot map uio device".to_string()))?;
    Ok(mem as *const hchainio0::RegisterBlock)
}

/// Performs IRQ mapping of IP core's block
fn map_irq(hashboard_idx: usize, irq_type: &'static str) -> error::Result<uio::UioDevice> {
    let uio_name = format!("chain{}-{}", hashboard_idx - 1, irq_type);
    let uio = uio::UioDevice::open_by_name(&uio_name)?;
    Ok(uio)
}

/// This is common implementation
impl<'a> HChainFifo<'a> {
    #[inline]
    pub fn is_work_tx_fifo_full(&self) -> bool {
        self.hash_chain_io.stat_reg.read().work_tx_full().bit()
    }

    #[inline]
    pub fn is_work_rx_fifo_empty(&self) -> bool {
        self.hash_chain_io.stat_reg.read().work_rx_empty().bit()
    }

    #[inline]
    pub fn is_cmd_rx_fifo_empty(&self) -> bool {
        self.hash_chain_io.stat_reg.read().cmd_rx_empty().bit()
    }

    /// Wait fro command FIFO to become empty.
    /// Uses polling.
    pub fn wait_cmd_tx_fifo_empty(&self) {
        // TODO busy waiting has to be replaced once asynchronous processing is in place
        // jho: Not really, there's no IRQ for cmd tx fifo becomming "empty". The best we
        // can do is run this in a separate thread with timeout polling.
        // But we usually want to wait for cmd to be empty before we issue other commands,
        // so it's not really worth it to pursue asynchronicity vehemently in this case.
        while !self.hash_chain_io.stat_reg.read().cmd_tx_empty().bit() {}
    }

    pub fn enable_ip_core(&self) {
        self.hash_chain_io
            .ctrl_reg
            .modify(|_, w| w.enable().bit(true));
    }

    pub fn disable_ip_core(&self) {
        self.hash_chain_io
            .ctrl_reg
            .modify(|_, w| w.enable().bit(false));
    }

    pub fn set_ip_core_work_time(&self, work_time: u32) {
        self.hash_chain_io
            .work_time
            .write(|w| unsafe { w.bits(work_time) });
    }

    pub fn set_baud_clock_div(&self, baud_clock_div: u32) {
        self.hash_chain_io
            .baud_reg
            .write(|w| unsafe { w.bits(baud_clock_div) });
    }

    pub fn set_ip_core_midstate_count(&self, count: u8) {
        self.hash_chain_io
            .ctrl_reg
            .modify(|_, w| unsafe { w.midstate_cnt().bits(count) });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::io;

    /// Try opening UIO device.
    /// This test needs properly configured UIO devices for hash-chain 8 in
    /// device-tree so that we have something to open.
    #[test]
    fn test_lookup_uio() {
        let name = String::from("chain7-mem");
        uio::UioDevice::open_by_name(&name).unwrap();
    }

    /// Try opening non-existent UIO device.
    #[test]
    fn test_lookup_uio_notfound() {
        let name = String::from("chain7-nonsense");
        let uio = uio::UioDevice::open_by_name(&name);
        assert!(
            uio.is_err(),
            "Found UIO device {} that shouldn't really be there"
        );
    }

    /// Try mapping memory from UIO device.
    #[test]
    fn test_map_uio() {
        mmap(8).unwrap();
    }

    /// Try to map memory twice.
    /// This is to check that the UioMapping Drop trait is working: Drop
    /// does perform unmap which drops the Uio fd lock.
    #[test]
    fn test_map_uio_twice_checklock() {
        mmap(8).unwrap();
        mmap(8).unwrap();
    }

    /// Try to map IRQ.
    #[test]
    fn test_map_irq() {
        map_irq(8, "cmd-rx").unwrap();
    }

    /// Test that we get IRQ.
    /// Test it on empty tx queue (IRQ always asserted).
    #[test]
    fn test_get_irq() {
        let irq = map_irq(8, "work-tx").unwrap();
        irq.irq_enable().unwrap();
        let res = irq.irq_wait_timeout(FIFO_READ_TIMEOUT);
        assert!(res.unwrap().is_some(), "expected interrupt");
    }

    /// Test that we get timeout when waiting for IRQ.
    /// Test it on empty rx queue (IRQ always deasserted).
    #[test]
    fn test_get_irq_timeout() {
        let irq = map_irq(8, "work-rx").unwrap();
        irq.irq_enable().unwrap();
        let res = irq.irq_wait_timeout(FIFO_READ_TIMEOUT);
        assert!(res.unwrap().is_none(), "expected timeout");
    }
}
