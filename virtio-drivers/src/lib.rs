// SPDX-License-Identifier: MIT

//! VirtIO guest drivers.
//!
//! These drivers can be used by bare-metal code (such as a bootloader or OS kernel) running in a VM
//! to interact with VirtIO devices provided by the VMM (such as QEMU or crosvm).
//!
//! # Usage
//!
//! You must first implement the [`Hal`] trait, to allocate DMA regions and translate between
//! physical addresses (as seen by devices) and virtual addresses (as seen by your program). You can
//! then construct the appropriate transport for the VirtIO device, e.g. for an MMIO device (perhaps
//! discovered from the device tree):
//! EXAMPLES REMOVED TEMPORARILY
#![cfg_attr(not(test), no_std)]
#![deny(unused_must_use, missing_docs)]
#![allow(clippy::identity_op)]
#![allow(dead_code)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod device;
mod hal;
mod queue;
pub mod transport;
mod volatile;

use core::{
    fmt::{self, Display, Formatter},
    ptr::{self, NonNull},
};

pub use self::hal::{BufferDirection, Hal, PhysAddr};

/// The page size in bytes supported by the library (4 KiB).
pub const PAGE_SIZE: usize = 0x1000;

/// The type returned by driver methods.
pub type Result<T = ()> = core::result::Result<T, Error>;

/// The error type of VirtIO drivers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// There are not enough descriptors available in the virtqueue, try again later.
    QueueFull,
    /// The device is not ready.
    NotReady,
    /// The device used a different descriptor chain to the one we were expecting.
    WrongToken,
    /// The queue is already in use.
    AlreadyUsed,
    /// Invalid parameter.
    InvalidParam,
    /// Failed to alloc DMA memory.
    DmaError,
    /// I/O Error
    IoError,
    /// The request was not supported by the device.
    Unsupported,
    /// The config space advertised by the device is smaller than the driver expected.
    ConfigSpaceTooSmall,
    /// The device doesn't have any config space, but the driver expects some.
    ConfigSpaceMissing,
}

#[cfg(feature = "alloc")]
impl From<alloc::string::FromUtf8Error> for Error {
    fn from(_value: alloc::string::FromUtf8Error) -> Self {
        Self::IoError
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::QueueFull => write!(f, "Virtqueue is full"),
            Self::NotReady => write!(f, "Device not ready"),
            Self::WrongToken => write!(
                f,
                "Device used a different descriptor chain to the one we were expecting"
            ),
            Self::AlreadyUsed => write!(f, "Virtqueue is already in use"),
            Self::InvalidParam => write!(f, "Invalid parameter"),
            Self::DmaError => write!(f, "Failed to allocate DMA memory"),
            Self::IoError => write!(f, "I/O Error"),
            Self::Unsupported => write!(f, "Request not supported by device"),
            Self::ConfigSpaceTooSmall => write!(
                f,
                "Config space advertised by the device is smaller than expected"
            ),
            Self::ConfigSpaceMissing => {
                write!(
                    f,
                    "The device doesn't have any config space, but the driver expects some"
                )
            }
        }
    }
}

/// Align `size` up to a page.
fn align_up(size: usize) -> usize {
    (size + PAGE_SIZE) & !(PAGE_SIZE - 1)
}

/// The number of pages required to store `size` bytes, rounded up to a whole number of pages.
fn pages(size: usize) -> usize {
    size.div_ceil(PAGE_SIZE)
}

// TODO: Use NonNull::slice_from_raw_parts once it is stable.
/// Creates a non-null raw slice from a non-null thin pointer and length.
fn nonnull_slice_from_raw_parts<T>(data: NonNull<T>, len: usize) -> NonNull<[T]> {
    NonNull::new(ptr::slice_from_raw_parts_mut(data.as_ptr(), len)).unwrap()
}
