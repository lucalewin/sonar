// use crate::utils::ui_logger::ui_log;

#[cfg(target_os = "windows")]
pub fn raise_priority() {
    use log::{debug, error};
    use windows::Win32::{Foundation::GetLastError, System::Threading::*};
    unsafe {
        let id = GetCurrentProcess();
        if !SetPriorityClass(id, ABOVE_NORMAL_PRIORITY_CLASS).as_bool() {
            let e = GetLastError();
            let p = GetCurrentProcessId();
            error!("Failed to set process priority id={p}, error={e:?}");
        }
    }
    debug!("Now running at ABOVE_NORMAL_PRIORITY_CLASS");
}

#[cfg(target_os = "linux")]
pub fn raise_priority() {
    // the following only works when you're root on Linux
    // or if you give the program CAP_SYS_NICE (cf. setcap)
    use libc::{getpriority, setpriority, PRIO_PROCESS};
    unsafe {
        let pri = getpriority(PRIO_PROCESS, 0);
        let newpri = pri - 5;
        let rc = setpriority(PRIO_PROCESS, 0, newpri);
        if rc != 0 {
            ui_log("Sorry, but you don't have permissions to raise priority...".to_string());
        } else {
            ui_log(format!("Now running at nice value {newpri}"));
        }
    }
}

#[cfg(target_os = "macos")]
pub fn raise_priority() {}
