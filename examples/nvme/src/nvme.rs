use nix::request_code_readwrite;

pub type __u8 = std::os::raw::c_uchar;
pub type __u16 = std::os::raw::c_ushort;
pub type __u32 = std::os::raw::c_uint;
pub type __u64 = std::os::raw::c_ulonglong;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvme_uring_cmd {
    pub opcode: __u8,
    pub flags: __u8,
    pub rsvd1: __u16,
    pub nsid: __u32,
    pub cdw2: __u32,
    pub cdw3: __u32,
    pub metadata: __u64,
    pub addr: __u64,
    pub metadata_len: __u32,
    pub data_len: __u32,
    pub cdw10: __u32,
    pub cdw11: __u32,
    pub cdw12: __u32,
    pub cdw13: __u32,
    pub cdw14: __u32,
    pub cdw15: __u32,
    pub timeout_ms: __u32,
    pub rsvd2: __u32,
}

const IOC_NR_BITS: usize = 8;
const IOC_TYPE_BITS: usize = 8;
const IOC_SIZE_BITS: usize = 14;
const IOC_NR_SHIFT: usize = 0;
const IOC_READ: usize = 2;
const IOC_WRITE: usize = 2;

const IOC_TYPE_SHIFT: usize = IOC_NR_SHIFT + IOC_NR_BITS;
const IOC_SIZE_SHIFT: usize = IOC_TYPE_SHIFT + IOC_TYPE_BITS;
const IOC_DIRSHIFT: usize = IOC_SIZE_SHIFT + IOC_SIZE_BITS;

fn ioc(dir: usize, t: usize, nr: usize, size: usize) -> usize {
    (dir << IOC_DIRSHIFT) | (t << IOC_TYPE_SHIFT) | (nr << IOC_NR_SHIFT) | (size << IOC_SIZE_SHIFT)
}

fn iowr(t: usize, nr: usize, size: usize) -> usize {
    ioc(IOC_READ | IOC_WRITE, t, nr, size)
}

pub fn nvme_uring_cmd_io() -> u32 {
    iowr('N' as usize, 0x80, core::mem::size_of::<nvme_uring_cmd>()) as u32
    //iowr('N' as usize, 0x80, 32) as u32
}

pub const NVME_URING_CMD_IO: u32 =
    request_code_readwrite!('N', 0x80, core::mem::size_of::<nvme_uring_cmd>()) as u32;
