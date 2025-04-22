use std::error::Error;

#[macro_export]
macro_rules! log {
    ($log:ident, $($arg:tt)*) => {{
        let _ = $log.log(format!($($arg)*));
    }}
}

pub trait Logger {
    fn log(&self, message: impl ToString) -> Result<(), Box<dyn Error>>;
}

pub struct StdOutLogger;

impl Logger for &StdOutLogger {
    fn log(&self, message: impl ToString) -> Result<(), Box<dyn Error>> {
        println!("{}", message.to_string());
        Result::Ok(())
    }
}

impl Logger for StdOutLogger {
    fn log(&self, message: impl ToString) -> Result<(), Box<dyn Error>> {
        (&self).log(message)
    }
}
