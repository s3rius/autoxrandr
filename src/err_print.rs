use std::fmt::Display;

pub trait ErrPrint {
    fn err_print(self, msg: String) -> Self;
}

impl<I, T: Display> ErrPrint for Result<I, T> {
    fn err_print(self, msg: String) -> Self {
        if let Err(err) = &self {
            eprintln!("{msg}. Cause: {err}");
        }
        self
    }
}

impl<T> ErrPrint for Option<T> {
    fn err_print(self, msg: String) -> Self {
        if self.is_none() {
            eprintln!("{msg}. Cause the value is none.");
        }
        self
    }
}
