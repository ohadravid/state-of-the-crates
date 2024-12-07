use anyhow::Context;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyLibError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("got an invalid code `{0}`")]
    InvalidCode(u32),
}

fn my_lib_function() -> Result<u32, MyLibError> {
    let hosts = std::fs::read_to_string("/etc/hosts")?;
    Err(MyLibError::InvalidCode(42))
}

fn my_app_function() -> Result<(), anyhow::Error> {
    let res = my_lib_function();

    let res = match res {
        Err(MyLibError::IoError(_)) => todo!("implement retry logic"),
        res => res,
    };

    let _valid_code = res.context("Failed to get a valid code from my_lib_function")?;

    Ok(())
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    my_app_function().unwrap();

    Ok(())
}
