macro_rules! syn_error {
    ($span:expr, $msg:literal $(, $($arg:tt)+)?) => {
        syn::Error::new($span, format!($msg, $($($arg)+)?))
    }
}

pub(crate) use syn_error;

macro_rules! syn_bail {
    ($($arg:tt)+) => {
        return Err($crate::utils::error::syn_error!($($arg)+))
    }
}
pub(crate) use syn_bail;
