
pub enum DebuggerCatch {
    Handle(String),
    Panic(String),
}

#[macro_export]
macro_rules! MB {
    ($mbytes: expr) => {
        1024 * 1024 * $mbytes
    };
}

#[macro_export]
macro_rules! KB {
    ($kbytes: expr) => {
        1024 * $mbytes
    };
}

/// Usage: Pass a boolean expression as the first argument that must hold. If it does not
/// This will raise a SIGTRAP signal, thus telling any connected debugger to set a breakpoint immediately
/// and we get to analyze what's going on.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debugger_catch {
    ($assert_expr:expr, $message:literal) => {
        if !$assert_expr {
            println!("Assert failed - {} @ {}:{}:{}", $message, file!(), line!(), column!());
            unsafe { 
                let res = libc::raise(libc::SIGTRAP);
                if res != 0 {
                    println!("Error sending SIGTRAP signal. Debugger will not be notified (probably). System error message:{}", crate::utils::get_sys_error().unwrap());
                } else { 
                    println!("Reached stoppable debug statement");
                }
            }
        }
    };

    ($assert_expr:expr, $handleRequest:expr) => {
        let (file, line, column) = (file!(), line!(), column!());
        if !$assert_expr {
            match $handleRequest {
                crate::DebuggerCatch::Handle(message) => {
                    println!("Assert failed - {} @ {}:{}:{}", message, file, line, column);
                    unsafe { 
                        let res = libc::raise(libc::SIGTRAP);
                        if res != 0 {
                            println!("Error sending SIGTRAP signal. Debugger will not be notified (probably). System error message:{}", crate::utils::get_sys_error().unwrap());
                        } else { 
                            println!("Reached stoppable debug statement");
                        }
                    }
                },
                crate::DebuggerCatch::Panic(message) => {
                    panic!("Assert failed - {} @ {}:{}:{}", message, file, line, column);
                },
            }
        }
    };
}

/// Conditionally compiled only for debug builds. This is the exact equivalent of using the #[cfg(debug_assertions)] attribute on a statement or scope
/// but it's wrapped in a macro here, to signal intent more clearly to the reader
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! only_in_debug {
    ($e:expr) => {
        $e
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! only_in_debug {
    ($e:expr) => {
        
    };
}

#[macro_export]
macro_rules! diff {
    ($a:expr, $b:expr) => {
        ($a as i64 - $b as i64).abs() as usize
    };
}

/// Empty macro statement, so that our debugger_catch!() calls don't get compiled into the release binary. that would be completely
/// unnecessary
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debugger_catch {
    ($assert_expr:expr, $message:literal) => {};
    ($assert_expr:expr, $handleRequest:expr) => {};
}

#[macro_export]
macro_rules! Assert {
    ($assert_expr:expr, $message:literal) => {
        if !$assert_expr {
            println!("Assert failed - {} @ {}:{}:{}", $message, file!(), line!(), column!());
            unsafe { 
                let res = libc::raise(libc::SIGTRAP);
                if res != 0 {
                    panic!("Error sending SIGTRAP signal. Debugger will not be notified (probably). System error message:{}", crate::utils::get_sys_error().unwrap());
                } else { 
                    println!("Reached stoppable debug statement");
                }
            }
        }
    };

    ($assert_expr:expr, $message:expr) => {
        let (file, line, column) = (file!(), line!(), column!());
        if !$assert_expr {
            println!("Assert failed - {} @ {}:{}:{}", $message, file, line, column);
            unsafe { 
                let res = libc::raise(libc::SIGTRAP);
                if res != 0 {
                    panic!("Error sending SIGTRAP signal. Debugger will not be notified (probably). System error message:{}", crate::utils::get_sys_error().unwrap());
                } else { 
                    println!("Reached stoppable debug statement");
                }
            }
        }
    };
}

/// Macro that we use for convenience purposes for our "indexing" types, Index, Line, Column and Length
/// This was a decision I made, when I kept using the wrong usize numbers as parameters to the *many* functions that I have written
/// that takes these usize numbers. Now the compiler will tell me I'm a fucking moron when trying to use the wrong parameters.
#[macro_export]
macro_rules! IndexingType {
    ($(#[$attr:meta])*, $safe_type:ident, $wrapped_type:ty) => {
        #[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, serde::Serialize, serde::Deserialize)]
        $(#[$attr])*
        pub struct $safe_type(pub $wrapped_type);
        impl std::ops::Deref for $safe_type {
            type Target = $wrapped_type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::AddAssign for $safe_type {
            fn add_assign(&mut self, rhs: Self) {
                #[cfg(debug_assertions)]
                let copy = {
                    let $safe_type(copy) = self;
                    *copy
                };

                {
                    let $safe_type(ref mut this) = self;
                    let $safe_type(that) = rhs;
                    *this += that;
                }
                #[cfg(debug_assertions)] {
                    let $safe_type(this) = self;
                    debug_assert!(copy != *this, "value must change!!");
                }
            }
        }

        impl std::ops::Add for $safe_type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(*self + *rhs)
            }
        }

        impl std::ops::Sub for $safe_type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                let Self(this) = self;
                let Self(that) = rhs;
                Self(this - that)
            }
        }

        impl std::ops::SubAssign for $safe_type {
            fn sub_assign(&mut self, rhs: Self) {
                let $safe_type(ref mut this) = self;
                let $safe_type(that) = rhs;
                *this -= that;
            }
        }

        impl $safe_type {
            /// Takes the Index and adds the offset provided as parameter. If the result is negative
            /// the returned Index is Index(0). Thus, this type is always safe to add an offset to, that's negative
            /// This is the *absolute* best reason for wrapping usize and primitive types into a struct like this.
            pub fn offset(&self, offset: isize) -> Self {
                let Self(value) = self;
                let value = *value as isize;
                let result = value + offset;
                if result < 0 {
                    Self(0)
                } else {
                    Self(result as usize)
                }
            }

            pub fn offset_mut(&mut self, offset: isize) -> Self {
                let Self(value) = self;
                let value = *value as isize;
                let result = value + offset;
                if result < 0 {
                    Self(0)
                } else {
                    Self(result as usize)
                }
            }
        }

        impl Step for $safe_type {
            fn steps_between(start: &Self, end: &Self) -> Option<usize> {
                if start > end {
                    None
                } else {
                    Some(**end - **start)
                }
            }
        
            fn forward_checked(start: Self, count: usize) -> Option<Self> {
                Some(start.offset(count as isize))
            }
        
            fn backward_checked(start: Self, count: usize) -> Option<Self> {
                if count > *start {
                    None
                } else {
                    let offset = count as isize;
                    Some(start.offset(-1 * offset))
                }
            }
        }
    };
}

#[cfg(test)]
pub mod macro_tests {
    #[test]
    pub fn test_equivalent_functionality_macro_and_fn() {
        let v = vec!['f', 'o', 'o'];
        let s = "hello world";
        let fn_res = crate::utils::difference(v.len(), s.len());
        let macro_res = diff!(v.len(), s.len());
        assert_eq!(fn_res, macro_res);
    }
}
