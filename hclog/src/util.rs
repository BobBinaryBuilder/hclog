use crate::{
    Result,
    error::ErrorKind::EnvType,
};
use std::{
    env,
    str::FromStr,
};

// some crate local helpers
pub (in crate) fn read_var_from_env<T>(key: &str) -> Result<Option<T>>
where
    T: FromStr + Sized,
{
    if let Some(ref var) = env::var_os(key) {
        if let Some(v) = var.to_str() {
            match v.parse::<T>() {
                Ok(v) => return Ok(Some(v)),
                Err(_) => {
                    return Err(EnvType);
                }
            }
        }
    }
    Ok(None::<T>)
}


