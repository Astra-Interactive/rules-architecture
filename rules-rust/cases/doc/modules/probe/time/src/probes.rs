pub(crate) fn probe_chrono_utc_now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub(crate) fn probe_chrono_local_now() -> i64 {
    chrono::Local::now().timestamp()
}

pub(crate) fn probe_jiff_timestamp_now() -> jiff::Timestamp {
    jiff::Timestamp::now()
}

pub(crate) fn probe_jiff_zoned_now() -> jiff::Zoned {
    jiff::Zoned::now()
}

pub(crate) fn probe_time_offset_date_time_now_utc() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

pub(crate) fn probe_time_offset_date_time_now_local() -> bool {
    time::OffsetDateTime::now_local().is_ok()
}

pub(crate) fn probe_time_utc_date_time_now() -> time::UtcDateTime {
    time::UtcDateTime::now()
}

pub(crate) fn probe_deprecated_chrono_utc_today() -> chrono::Date<chrono::Utc> {
    chrono::Utc::today()
}

pub(crate) fn probe_deprecated_chrono_local_today() -> chrono::Date<chrono::Local> {
    chrono::Local::today()
}
