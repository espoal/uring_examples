// Questions:
// 1. How do I setup large queues? and in general pass flags to the ring?
// 2. Is buff done correctly?
// 3. Does it looks right?

pub type __u8 = core::std::os::raw::c_uchar;
pub type __u16 = core::std::os::raw::c_ushort;
pub type __u32 = core::std::os::raw::c_uint;
pub type __u64 = core::std::os::raw::c_ulonglong;

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