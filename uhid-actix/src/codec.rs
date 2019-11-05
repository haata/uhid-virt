use std::convert::TryFrom;
use std::io;
use std::mem;
use std::slice;

use arrayvec::{ArrayString, ArrayVec};

use uhid_sys as sys;

pub enum StreamError {
    Io { err: io::Error },
    UnknownEventType { event_type_value: u32 },
    BufferOverflow { data_size: usize, max_size: usize },
    Unknown,
}

struct DevFlags {
    flags: u64,
}

impl DevFlags {
    // const NUMBERED_FEATURE_REPORTS: DevFlags = 0b0000_0001;
    // const NUMBERED_OUTPUT_REPORTS: DevFlags = 0b0000_0010;
    // const NUMBERED_INPUT_REPORTS: DevFlags = 0b0000_0100;
    fn feature_reports_numbered(&self) -> bool {
        self.flags % 2 == 1
    }

    fn output_reports_numbered(&self) -> bool {
        (self.flags >> 1) % 2 == 1
    }

    fn input_reports_numbered(&self) -> bool {
        (self.flags >> 2) % 2 == 1
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ReportType {
    Feature = 0,
    Output = 1,
    Input = 2,
}

#[allow(non_camel_case_types)]
pub enum Bus {
    PCI = 1,
    ISAPNP = 2,
    USB = 3,
    HIL = 4,
    BLUETOOTH = 5,
    VIRTUAL = 6,
    ISA = 16,
    I8042 = 17,
    XTKBD = 18,
    RS232 = 19,
    GAMEPORT = 20,
    PARPORT = 21,
    AMIGA = 22,
    ADB = 23,
    I2C = 24,
    HOST = 25,
    GSC = 26,
    ATARI = 27,
    SPI = 28,
    RMI = 29,
    CEC = 30,
    INTEL_ISHTP = 31,
}

pub enum InputEvent {
    Create {
        name: ArrayString<[u8; 128]>,
        phys: ArrayString<[u8; 64]>,
        uniq: ArrayString<[u8; 64]>,
        bus: Bus,
        vendor: u32,
        product: u32,
        version: u32,
        country: u32,
        rd_data: ArrayVec<[u8; sys::HID_MAX_DESCRIPTOR_SIZE as usize]>,
    },
    Destroy,
    Input {
        data: ArrayVec<[u8; sys::UHID_DATA_MAX as usize]>,
    },
    GetReportReply {
        id: u32,
        err: u16,
        data: ArrayVec<[u8; sys::UHID_DATA_MAX as usize]>,
    },
    SetReportReply {
        id: u32,
        err: u16,
    },
}

impl Into<sys::uhid_event> for InputEvent {
    fn into(self) -> sys::uhid_event {
        let mut event: sys::uhid_event = unsafe { mem::zeroed() };

        match self {
            InputEvent::Create {
                name,
                phys,
                uniq,
                bus,
                vendor,
                product,
                version,
                country,
                rd_data,
            } => {
                event.type_ = sys::uhid_event_type_UHID_CREATE2 as u32;
                let payload = unsafe { &mut event.u.create2 };
                name.as_bytes()
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.name[i] = *x);
                phys.as_bytes()
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.phys[i] = *x);
                uniq.as_bytes()
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.uniq[i] = *x);
                rd_data
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.rd_data[i] = *x);
                payload.rd_size = rd_data.len() as u16;
                payload.bus = bus as u16;
                payload.vendor = vendor;
                payload.product = product;
                payload.version = version;
                payload.country = country;
            }
            InputEvent::Destroy => {
                event.type_ = sys::uhid_event_type_UHID_DESTROY as u32;
            }
            InputEvent::Input { data } => {
                event.type_ = sys::uhid_event_type_UHID_INPUT2 as u32;
                let payload = unsafe { &mut event.u.input2 };
                data.iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.data[i] = *x);
                payload.size = data.len() as u16;
            }
            InputEvent::GetReportReply { err, data, .. } => {
                event.type_ = sys::uhid_event_type_UHID_GET_REPORT_REPLY as u32;
                let payload = unsafe { &mut event.u.get_report_reply };
                payload.err = err;
                data.iter()
                    .enumerate()
                    .for_each(|(i, x)| payload.data[i] = *x);
                payload.size = data.len() as u16;
            }
            InputEvent::SetReportReply { err, .. } => {
                event.type_ = sys::uhid_event_type_UHID_SET_REPORT_REPLY as u32;
                let payload = unsafe { &mut event.u.set_report_reply };
                payload.err = err;
            }
        };

        event
    }
}

pub enum OutputEvent {
    Start {
        dev_flags: DevFlags,
    },
    Stop,
    Open,
    Close,
    Output {
        data: Vec<u8>,
    },
    GetReport {
        id: u32,
        report_number: u8,
        report_type: ReportType,
    },
    SetReport {
        id: u32,
        report_number: u8,
        report_type: ReportType,
        data: Vec<u8>,
    },
}

impl TryFrom<sys::uhid_event> for OutputEvent {
    type Error = u32;
    fn try_from(event: sys::uhid_event) -> Result<OutputEvent, u32> {
        if let Some(event_type) = to_uhid_event_type(event.type_) {
            match event_type {
                sys::uhid_event_type_UHID_START => Ok(unsafe {
                    let payload = &event.u.start;
                    OutputEvent::Start {
                        dev_flags: mem::transmute(payload.dev_flags),
                    }
                }),
                sys::uhid_event_type_UHID_STOP => Ok(OutputEvent::Stop),
                sys::uhid_event_type_UHID_OPEN => Ok(OutputEvent::Open),
                sys::uhid_event_type_UHID_CLOSE => Ok(OutputEvent::Close),
                sys::uhid_event_type_UHID_OUTPUT => Ok(unsafe {
                    let payload = &event.u.output;
                    assert_eq!(
                        payload.rtype,
                        sys::uhid_report_type_UHID_OUTPUT_REPORT as u8
                    );
                    OutputEvent::Output {
                        data: slice::from_raw_parts(
                            &payload.data[0] as *const u8,
                            payload.size as usize,
                        )
                        .to_vec(),
                    }
                }),
                sys::uhid_event_type_UHID_GET_REPORT => Ok(unsafe {
                    let payload = &event.u.get_report;
                    OutputEvent::GetReport {
                        id: payload.id,
                        report_number: payload.rnum,
                        report_type: mem::transmute(payload.rtype),
                    }
                }),
                sys::uhid_event_type_UHID_SET_REPORT => Ok(unsafe {
                    let payload = &event.u.set_report;
                    OutputEvent::SetReport {
                        id: payload.id,
                        report_number: payload.rnum,
                        report_type: mem::transmute(payload.rtype),
                        data: slice::from_raw_parts(
                            &payload.data[0] as *const u8,
                            payload.size as usize,
                        )
                        .to_vec(),
                    }
                }),
                _ => Err(event.type_),
            }
        } else {
            Err(event.type_)
        }
    }
}

fn encode_event(event: &sys::uhid_event) -> &[u8] {
    unsafe { as_u8_slice(event) }
}

unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    slice::from_raw_parts((p as *const T) as *const u8, mem::size_of::<T>())
}

fn to_uhid_event_type(value: u32) -> Option<sys::uhid_event_type> {
    let last_valid_value = sys::uhid_event_type_UHID_SET_REPORT_REPLY as u32;
    if value <= last_valid_value {
        Some(value)
    } else {
        None
    }
}

fn read_event(src: &mut [u8]) -> Option<sys::uhid_event> {
    let uhid_event_size = mem::size_of::<sys::uhid_event>();
    if src.len() >= uhid_event_size {
        let bytes = Vec::from(&src[..=uhid_event_size]);
        let ptr = bytes.as_ptr();
        Some(unsafe { *(ptr as *const sys::uhid_event) })
    } else {
        None
    }
}

// TODO: Use a legit error here
impl TryFrom<Vec<u8>> for OutputEvent {
    type Error = u32;
    fn try_from(mut vec: Vec<u8>) -> Result<Self, Self::Error> {
        if let Some(event) = read_event(&mut vec) {
            OutputEvent::try_from(event)
        } else {
            Err(0)
        }
    }

    // fn read_len(&self) -> usize {
    //     mem::size_of::<sys::uhid_event>()
    // }
}

impl Into<Vec<u8>> for InputEvent {
    fn into(self) -> Vec<u8> {
        let event = self.into();
        Vec::from(encode_event(&event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RDESC: [u8; 85] = [
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x02, /* USAGE (Mouse) */
        0xa1, 0x01, /* COLLECTION (Application) */
        0x09, 0x01, /* USAGE (Pointer) */
        0xa1, 0x00, /* COLLECTION (Physical) */
        0x85, 0x01, /* REPORT_ID (1) */
        0x05, 0x09, /* USAGE_PAGE (Button) */
        0x19, 0x01, /* USAGE_MINIMUM (Button 1) */
        0x29, 0x03, /* USAGE_MAXIMUM (Button 3) */
        0x15, 0x00, /* LOGICAL_MINIMUM (0) */
        0x25, 0x01, /* LOGICAL_MAXIMUM (1) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x75, 0x01, /* REPORT_SIZE (1) */
        0x81, 0x02, /* INPUT (Data,Var,Abs) */
        0x95, 0x01, /* REPORT_COUNT (1) */
        0x75, 0x05, /* REPORT_SIZE (5) */
        0x81, 0x01, /* INPUT (Cnst,Var,Abs) */
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x30, /* USAGE (X) */
        0x09, 0x31, /* USAGE (Y) */
        0x09, 0x38, /* USAGE (WHEEL) */
        0x15, 0x81, /* LOGICAL_MINIMUM (-127) */
        0x25, 0x7f, /* LOGICAL_MAXIMUM (127) */
        0x75, 0x08, /* REPORT_SIZE (8) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x81, 0x06, /* INPUT (Data,Var,Rel) */
        0xc0, /* END_COLLECTION */
        0xc0, /* END_COLLECTION */
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x06, /* USAGE (Keyboard) */
        0xa1, 0x01, /* COLLECTION (Application) */
        0x85, 0x02, /* REPORT_ID (2) */
        0x05, 0x08, /* USAGE_PAGE (Led) */
        0x19, 0x01, /* USAGE_MINIMUM (1) */
        0x29, 0x03, /* USAGE_MAXIMUM (3) */
        0x15, 0x00, /* LOGICAL_MINIMUM (0) */
        0x25, 0x01, /* LOGICAL_MAXIMUM (1) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x75, 0x01, /* REPORT_SIZE (1) */
        0x91, 0x02, /* Output (Data,Var,Abs) */
        0x95, 0x01, /* REPORT_COUNT (1) */
        0x75, 0x05, /* REPORT_SIZE (5) */
        0x91, 0x01, /* Output (Cnst,Var,Abs) */
        0xc0, /* END_COLLECTION */
    ];

    fn assert_bytes_eq(actual: &[u8], expected: &[u8]) {
        assert_eq!(actual.len(), expected.len(), "Size of slices differs");
        for index in 0..actual.len() {
            assert_eq!(
                actual[index], expected[index],
                "Bytes differ at index {}",
                index
            );
        }
    }

    #[test]
    fn encode_create_request() {
        let mut expected = vec![0; mem::size_of::<sys::uhid_event>()];
        expected[0] = 0x0b;
        expected[4] = 0x74;
        expected[5] = 0x65;
        expected[6] = 0x73;
        expected[7] = 0x74;
        expected[8] = 0x2d;
        expected[9] = 0x75;
        expected[10] = 0x68;
        expected[11] = 0x69;
        expected[12] = 0x64;
        expected[13] = 0x2d;
        expected[14] = 0x64;
        expected[15] = 0x65;
        expected[16] = 0x76;
        expected[17] = 0x69;
        expected[18] = 0x63;
        expected[19] = 0x65;
        expected[260] = 0x55;
        expected[262] = 0x03;
        expected[264] = 0xd9;
        expected[265] = 0x15;
        expected[268] = 0x37;
        expected[269] = 0x0a;
        expected[280] = 0x05;
        expected[281] = 0x01;
        expected[282] = 0x09;
        expected[283] = 0x02;
        expected[284] = 0xa1;
        expected[285] = 0x01;
        expected[286] = 0x09;
        expected[287] = 0x01;
        expected[288] = 0xa1;
        expected[290] = 0x85;
        expected[291] = 0x01;
        expected[292] = 0x05;
        expected[293] = 0x09;
        expected[294] = 0x19;
        expected[295] = 0x01;
        expected[296] = 0x29;
        expected[297] = 0x03;
        expected[298] = 0x15;
        expected[300] = 0x25;
        expected[301] = 0x01;
        expected[302] = 0x95;
        expected[303] = 0x03;
        expected[304] = 0x75;
        expected[305] = 0x01;
        expected[306] = 0x81;
        expected[307] = 0x02;
        expected[308] = 0x95;
        expected[309] = 0x01;
        expected[310] = 0x75;
        expected[311] = 0x05;
        expected[312] = 0x81;
        expected[313] = 0x01;
        expected[314] = 0x05;
        expected[315] = 0x01;
        expected[316] = 0x09;
        expected[317] = 0x30;
        expected[318] = 0x09;
        expected[319] = 0x31;
        expected[320] = 0x09;
        expected[321] = 0x38;
        expected[322] = 0x15;
        expected[323] = 0x81;
        expected[324] = 0x25;
        expected[325] = 0x7f;
        expected[326] = 0x75;
        expected[327] = 0x08;
        expected[328] = 0x95;
        expected[329] = 0x03;
        expected[330] = 0x81;
        expected[331] = 0x06;
        expected[332] = 0xc0;
        expected[333] = 0xc0;
        expected[334] = 0x05;
        expected[335] = 0x01;
        expected[336] = 0x09;
        expected[337] = 0x06;
        expected[338] = 0xa1;
        expected[339] = 0x01;
        expected[340] = 0x85;
        expected[341] = 0x02;
        expected[342] = 0x05;
        expected[343] = 0x08;
        expected[344] = 0x19;
        expected[345] = 0x01;
        expected[346] = 0x29;
        expected[347] = 0x03;
        expected[348] = 0x15;
        expected[350] = 0x25;
        expected[351] = 0x01;
        expected[352] = 0x95;
        expected[353] = 0x03;
        expected[354] = 0x75;
        expected[355] = 0x01;
        expected[356] = 0x91;
        expected[357] = 0x02;
        expected[358] = 0x95;
        expected[359] = 0x01;
        expected[360] = 0x75;
        expected[361] = 0x05;
        expected[362] = 0x91;
        expected[363] = 0x01;
        expected[364] = 0xc0;

        let mut rd_data = ArrayVec::new();
        RDESC.iter().for_each(|x| rd_data.try_push(*x).unwrap());
        let result: Vec<u8> = InputEvent::Create {
            name: ArrayString::from("test-uhid-device").unwrap(),
            phys: ArrayString::from("").unwrap(),
            uniq: ArrayString::from("").unwrap(),
            bus: Bus::USB,
            vendor: 0x15d9,
            product: 0x0a37,
            version: 0,
            country: 0,
            rd_data,
        }
        .into();

        assert_bytes_eq(&result[..], &expected);
    }

    #[test]
    fn encode_destroy_request() {
        let mut expected = vec![0; mem::size_of::<sys::uhid_event>()];
        expected[0] = 0x01;

        let result: Vec<u8> = InputEvent::Destroy.into();
        assert_bytes_eq(&result[..], &expected);
    }
}
