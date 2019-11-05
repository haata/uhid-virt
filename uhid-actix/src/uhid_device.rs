use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use arrayvec::{ArrayString, ArrayVec};

use uhid_sys as sys;

use crate::codec::*;

pub struct UHIDDevice<T: Read + Write> {
    handle: T,
}

/// Parameters used to create UHID devices
#[derive(Debug, Clone, PartialEq)]
pub struct CreateParams {
    pub name: ArrayString<[u8; 128]>,
    pub phys: ArrayString<[u8; 64]>,
    pub uniq: ArrayString<[u8; 64]>,
    pub bus: Bus,
    pub vendor: u32,
    pub product: u32,
    pub version: u32,
    pub country: u32,
    pub rd_data: ArrayVec<[u8; sys::HID_MAX_DESCRIPTOR_SIZE as usize]>,
}

impl<T: Read + Write> UHIDDevice<T> {
    pub fn send_input(
        &mut self,
        data: ArrayVec<[u8; sys::UHID_DATA_MAX as usize]>,
    ) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Input { data }.into();
        self.handle.write(&event)
    }

    pub fn recv_output(&mut self) -> Result<OutputEvent, StreamError> {
        let mut event = [0; UHID_EVENT_SIZE];
        self.handle
            .read_exact(&mut event)
            .map_err(|err| StreamError::Io(err))?;
        OutputEvent::try_from(&event)
    }

    pub fn destroy(&mut self) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Destroy.into();
        self.handle.write(&event)
    }
}

impl UHIDDevice<File> {
    pub fn try_new(params: CreateParams) -> io::Result<UHIDDevice<File>> {
        UHIDDevice::try_new_with_path(params, Path::new("/dev/uhid"))
    }
    pub fn try_new_with_path(params: CreateParams, path: &Path) -> io::Result<UHIDDevice<File>> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        if cfg!(unix) {
            options.custom_flags(libc::O_RDWR | libc::O_CLOEXEC | libc::O_NONBLOCK);
        }
        let mut handle = options.open(path).unwrap();
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Create(params).into();
        handle.write(&event).unwrap();
        Ok(UHIDDevice { handle })
    }
}
