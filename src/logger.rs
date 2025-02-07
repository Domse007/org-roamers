use emacs::{Env, Result};

pub trait Logger {
    fn log(&self, message: impl ToString)-> Result<()>;
}

impl Logger for &Env {
    fn log(&self, message: impl ToString) -> Result<()>{
	self.message(message.to_string()).map(|_| ())
    }
}

pub struct StdOutLogger;

impl Logger for &StdOutLogger {
    fn log(&self, message: impl ToString) -> Result<()>{
	println!("{}", message.to_string());
	Result::Ok(())
    }
}
