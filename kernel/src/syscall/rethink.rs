use super::*;
use crate::arch::interrupt;
use crate::memory::{GlobalFrameAlloc, KernelStack};

impl Syscall<'_> {
    pub fn sys_get_syscall(&mut self) -> SysResult {
        info!("sys_get_syscall");
        let ret: extern "C" fn(tf: &mut TrapFrame) -> () = interrupt::syscall;
        Ok(ret as usize)
    }

    pub fn sys_kernel_fork(&mut self) -> SysResult {
        // let new_thread = self.thread.kernel_fork(self.tf);
        let new_thread = self.thread.fork(self.tf);
        let pid = new_thread.proc.lock().pid.get();
        let tid = processor().manager().add(new_thread);
        processor().manager().detach(tid);
        info!("fork: {} -> {}", thread::current().id(), pid);
        Ok(pid)
    }

    pub fn sys_vkernel_fork(&mut self) -> SysResult {
        self.sys_kernel_fork()
    }

    pub fn sys_kernel_exec(
        &mut self,
        path: *const u8,
        argv: *const *const u8,
        envp: *const *const u8,
    ) -> SysResult {
        info!(
            "exec:BEG: path: {:?}, argv: {:?}, envp: {:?}",
            path, argv, envp
        );
        let mut proc = self.process();
        let path = check_and_clone_cstr(path)?;
        info!("path check pass");
        let args = check_and_clone_cstr_array(argv)?;
        info!("args check pass");
        let envs = check_and_clone_cstr_array(envp)?;
        info!("envs check pass");

        if args.is_empty() {
            error!("exec: args is null");
            return Err(SysError::EINVAL);
        }

        info!(
            "exec:STEP2: path: {:?}, args: {:?}, envs: {:?}",
            path, args, envs
        );

        // Kill other threads
        proc.threads.retain(|&tid| {
            if tid != processor().tid() {
                processor().manager().exit(tid, 1);
            }
            tid == processor().tid()
        });

        // Read program file
        let inode = proc.lookup_inode(&path)?;

        // Make new Thread
        //let kstack = KernelStack::new();
        let (mut vm, entry_addr, ustack_top) =
            Thread::new_kernel_vm(&inode, &path, args, envs).map_err(|_| SysError::EINVAL)?;

        // Activate new page table
        core::mem::swap(&mut *self.vm(), &mut vm);
        unsafe {
            self.vm().activate();
        }

        // Modify exec path
        proc.exec_path = path.clone();
        drop(proc);

        // Modify the TrapFrame
        *self.tf = TrapFrame::new_kernel_thread_for_user(entry_addr, ustack_top);

        info!("exec:END: path: {:?}", path);
        Ok(0)
    }
}
