use crate::{pg_sys, FromDatum, IntoDatum};
use std::ops::{Deref, DerefMut};
use time::ComponentRangeError;

pub(crate) const USECS_PER_HOUR: i64 = 3600000000;
pub(crate) const USECS_PER_MINUTE: i64 = 60000000;
pub(crate) const USECS_PER_SEC: i64 = 1000000;
pub(crate) const MINS_PER_HOUR: i64 = 60;
pub(crate) const SEC_PER_MIN: i64 = 60;

pub struct Time(pub(crate) time::Time);
impl FromDatum<Time> for Time {
    #[inline]
    unsafe fn from_datum(datum: pg_sys::Datum, is_null: bool, _typoid: u32) -> Option<Time> {
        if is_null {
            None
        } else {
            let mut time = datum as i64;

            let hour = time / USECS_PER_HOUR;
            time -= hour * USECS_PER_HOUR;

            let min = time / USECS_PER_MINUTE;
            time -= min * USECS_PER_MINUTE;

            let second = time / USECS_PER_SEC;
            time -= second * USECS_PER_SEC;

            let microsecond = time;

            Some(Time(
                time::Time::try_from_hms_micro(
                    hour as u8,
                    min as u8,
                    second as u8,
                    microsecond as u32,
                )
                .expect("failed to convert time"),
            ))
        }
    }
}

impl IntoDatum<Time> for Time {
    #[inline]
    fn into_datum(self) -> Option<pg_sys::Datum> {
        let datum = ((((self.hour() as i64 * MINS_PER_HOUR + self.minute() as i64) * SEC_PER_MIN)
            + self.second() as i64)
            * USECS_PER_SEC)
            + self.microsecond() as i64;

        Some(datum as pg_sys::Datum)
    }
}

impl Time {
    pub fn new(time: time::Time) -> Self {
        Time(time)
    }

    pub fn from_hms(
        hour: u8,
        minute: u8,
        second: u8,
    ) -> std::result::Result<Time, ComponentRangeError> {
        match time::Time::try_from_hms(hour, minute, second) {
            Ok(time) => Ok(Time(time)),
            Err(e) => Err(e),
        }
    }
}

impl Deref for Time {
    type Target = time::Time;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Time {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl serde::Serialize for Time {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.format("%h-%m-%s"))
    }
}
