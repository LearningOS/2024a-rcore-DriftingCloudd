//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, 
    mm::{translated_byte_buffer, MapPermission, VirtAddr}
    ,
    task::{
        add_new_area_current, change_program_brk, current_user_token, 
        exit_current_and_run_next, get_current_memset, get_current_start_time, 
        get_current_syscall_times, suspend_current_and_run_next, TaskStatus,
        unmmap_area_current,
    }, 
    timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    write_from_kernel_to_user(ts, timeval);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let taskinfo =  {
        let start_time;
        if get_current_start_time() == 0
        {
            start_time = 0;
        } else {
            start_time = get_time_ms() - get_current_start_time();
        }

        TaskInfo {
            status: TaskStatus::Running,
            syscall_times: get_current_syscall_times(),
            time: start_time,
        }
    };
    write_from_kernel_to_user(_ti, taskinfo);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _port & !0x7 !=0 || 
    _port & 0x7 == 0 || 
    _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    let start_vaddr = VirtAddr::from(_start);
    let end_vaddr =VirtAddr::from( _start + _len);
    let current_memset = get_current_memset();

    if current_memset.area_overlap(start_vaddr, end_vaddr) == true {
        return -1;
    }

    let permission = turn_to_permission(_port);
    add_new_area_current(start_vaddr, end_vaddr, permission);
    
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    
    let start_vaddr = VirtAddr::from(_start);
    let end_vaddr = VirtAddr::from(_start + _len);
    if unmmap_area_current(start_vaddr, end_vaddr) == true
    {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

fn write_from_kernel_to_user<T>(dest: *mut T, src: T) {
    let buffer = translated_byte_buffer(
        current_user_token(), 
        dest as *mut u8,
        core::mem::size_of::<T>()
    );

    let src_ptr = &src as *const _ as *const u8;
    let src_bytes = 
        unsafe { core::slice::from_raw_parts(src_ptr, core::mem::size_of::<T>()) };
    let mut offset = 0;
    for buf in buffer {
        let len = buf.len().min(src_bytes.len() - offset);
        buf[..len].copy_from_slice(&src_bytes[offset..offset + len]);
        offset += len;
    }

}

fn turn_to_permission(prot: usize) -> MapPermission {
    let mut permission = MapPermission::empty();
    if prot & 0x1 != 0 {
        permission |= MapPermission::R;
    }
    if prot & 0x2 != 0 {
        permission |= MapPermission::W;
    }
    if prot & 0x4 != 0 {
        permission |= MapPermission::X;
    }
    permission | MapPermission::U
}
