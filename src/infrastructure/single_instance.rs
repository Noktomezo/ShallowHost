use std::fmt;

pub enum AcquireResult {
    Primary(SingleInstance),
    Secondary,
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::System::Threading::{
        CreateEventW, CreateMutexW, SetEvent, WaitForSingleObject,
    };

    use super::{AcquireResult, SingleInstanceError};

    struct OwnedHandle(HANDLE);

    impl OwnedHandle {
        fn new(handle: HANDLE) -> Option<Self> {
            (!handle.is_null()).then_some(Self(handle))
        }
    }

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: This handle was returned by CreateEventW/CreateMutexW, is owned by this
            // wrapper, and is closed exactly once when the wrapper is dropped.
            if unsafe { CloseHandle(self.0) } == 0 {
                eprintln!(
                    "failed to close a single-instance handle: {}",
                    std::io::Error::last_os_error()
                );
            }
        }
    }

    pub struct SingleInstance {
        _mutex: OwnedHandle,
        activation_event: OwnedHandle,
    }

    impl SingleInstance {
        pub fn acquire(app_id: &str) -> Result<AcquireResult, SingleInstanceError> {
            let event_name = wide_name(app_id, "Activate");
            let mutex_name = wide_name(app_id, "Mutex");

            // SAFETY: The names are valid, null-terminated UTF-16 buffers that remain alive for
            // each call. Default security and non-inheritable handles are sufficient here.
            let event = unsafe { CreateEventW(ptr::null(), 0, 0, event_name.as_ptr()) };
            let event = OwnedHandle::new(event)
                .ok_or_else(|| SingleInstanceError::CreateEvent(std::io::Error::last_os_error()))?;

            // SAFETY: The name is a valid, null-terminated UTF-16 buffer. The mutex is not
            // initially owned because its lifetime, rather than lock ownership, is the guard.
            let mutex = unsafe { CreateMutexW(ptr::null(), 0, mutex_name.as_ptr()) };
            if mutex.is_null() {
                return Err(SingleInstanceError::CreateMutex(
                    std::io::Error::last_os_error(),
                ));
            }
            // SAFETY: GetLastError is read immediately after the successful CreateMutexW call,
            // which documents ERROR_ALREADY_EXISTS as its existing-object result.
            let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
            let mutex = OwnedHandle::new(mutex)
                .ok_or_else(|| SingleInstanceError::CreateMutex(std::io::Error::last_os_error()))?;

            if already_running {
                // SAFETY: `event` is a live event handle created/opened above.
                if unsafe { SetEvent(event.0) } == 0 {
                    return Err(SingleInstanceError::Signal(std::io::Error::last_os_error()));
                }
                return Ok(AcquireResult::Secondary);
            }

            Ok(AcquireResult::Primary(Self {
                _mutex: mutex,
                activation_event: event,
            }))
        }

        pub fn activation_requested(&self) -> Result<bool, SingleInstanceError> {
            // SAFETY: `activation_event` remains valid for the lifetime of `self`; a zero timeout
            // makes this a non-blocking poll on GPUI's foreground thread.
            match unsafe { WaitForSingleObject(self.activation_event.0, 0) } {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                _ => Err(SingleInstanceError::Wait(std::io::Error::last_os_error())),
            }
        }
    }

    fn wide_name(app_id: &str, object: &str) -> Vec<u16> {
        OsStr::new(&format!(r"Local\{app_id}.Singleton.{object}"))
            .encode_wide()
            .chain(Some(0))
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::{SingleInstance, wide_name};
        use crate::infrastructure::single_instance::AcquireResult;

        #[test]
        fn development_and_release_use_different_kernel_objects() {
            assert_ne!(
                wide_name("Noktomezo.ShallowHost.Dev", "Mutex"),
                wide_name("Noktomezo.ShallowHost", "Mutex")
            );
        }

        #[test]
        fn secondary_instance_signals_the_primary() {
            let app_id = format!("Noktomezo.ShallowHost.SingletonTest.{}", std::process::id());
            let primary = match SingleInstance::acquire(&app_id) {
                Ok(AcquireResult::Primary(instance)) => instance,
                Ok(AcquireResult::Secondary) => panic!("test instance ID must be unique"),
                Err(error) => panic!("failed to create test instance: {error}"),
            };

            assert!(matches!(
                SingleInstance::acquire(&app_id),
                Ok(AcquireResult::Secondary)
            ));
            assert!(matches!(primary.activation_requested(), Ok(true)));
            assert!(matches!(primary.activation_requested(), Ok(false)));
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{AcquireResult, SingleInstanceError};

    pub struct SingleInstance;

    impl SingleInstance {
        pub fn acquire(_app_id: &str) -> Result<AcquireResult, SingleInstanceError> {
            Ok(AcquireResult::Primary(Self))
        }

        pub fn activation_requested(&self) -> Result<bool, SingleInstanceError> {
            Ok(false)
        }
    }
}

pub use platform::SingleInstance;

#[derive(Debug)]
pub enum SingleInstanceError {
    #[cfg(windows)]
    CreateEvent(std::io::Error),
    #[cfg(windows)]
    CreateMutex(std::io::Error),
    #[cfg(windows)]
    Signal(std::io::Error),
    #[cfg(windows)]
    Wait(std::io::Error),
}

impl fmt::Display for SingleInstanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(windows)]
            Self::CreateEvent(error) => {
                write!(formatter, "could not create activation event: {error}")
            }
            #[cfg(windows)]
            Self::CreateMutex(error) => {
                write!(formatter, "could not create instance mutex: {error}")
            }
            #[cfg(windows)]
            Self::Signal(error) => {
                write!(formatter, "could not signal the existing instance: {error}")
            }
            #[cfg(windows)]
            Self::Wait(error) => write!(formatter, "could not poll the activation event: {error}"),
        }
    }
}

impl std::error::Error for SingleInstanceError {}
