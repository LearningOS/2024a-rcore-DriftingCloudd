# Lab1 Report
## 编程作业
实现了以下功能
- 在`crate::task::TaskControlBlock`中增加了`time:usize`和`syscall_times: [u32; MAX_SYSCALL_NUM]`两个成员，分别用于记录应用开始时间与调用syscall次数
- 在`crate::task::TaskManger`中实现了`get_current_start_time()` `inc_current_syscall_times(syscal_nums: usize)`等接口
- 在`crate::task::TaskManager::run_next_task()`中，如果下个应用的开始时间`time == 0`，调用`set_current_start_time(time)`接口记录开始时间
- 在`crate::syscall::syscall`中，调用`inc_current_syscall_times(syscal_nums: usize)`接口，记录系统调用次数

## 简答作业
### 1
报错信息：
```bash
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.  
[kernel] IllegalInstruction in application, kernel killed it.  
[kernel] IllegalInstruction in application, kernel killed it.
```

即访问了非法地址，使用了非法指令，程序被系统杀死。
SBI 0.3.0-alpha.2

### 2
1. `a0`是当前进程`TrapContext`或者`TaskContext`的地址。`__restore`分别被用于**中断返回**和**调度下一个进程**
2. 处理了`sstatus` `spec` `sscratch`三个CSR寄存器。sstatus用于记录当前和进入S态之前的状态，spec用于记录`sret`后第一条指令，`sscratch`用来保存kernel stack top
3. `x2` `x4`分别是 `sp` `tp`。`sp`在后面保存，`tp`不需要保存
4. `sp`中是user stack top, `sscratch`是kernel stack top
5. L61 `sret`。执行`sret`系统硬件会进行一系列操作，如修改特权级
6. `sp`是kernel stack top, `sscratch`保存 user stack top
7. 在发生中断时硬件修改相关CSR寄存器后从U态进入S态