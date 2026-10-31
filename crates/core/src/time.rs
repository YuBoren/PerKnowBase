//! 时间：DB 存 epoch 毫秒，JSON 用 RFC 3339 UTC 毫秒（`contracts/README.md` §4）。
//!
//! 不引入 chrono/time：只需要一个方向的格式化，用公历换算即可，减少依赖与体积（NFR-09）。

use perknow_contracts::Timestamp;

/// 当前 epoch 毫秒。
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 当前时间的 RFC 3339 UTC 毫秒字符串，例 `2026-10-31T09:09:05.123Z`。
pub fn now_rfc3339_ms() -> Timestamp {
    to_rfc3339_ms(now_ms())
}

/// epoch 毫秒 → RFC 3339 UTC 毫秒字符串。
pub fn to_rfc3339_ms(epoch_ms: i64) -> Timestamp {
    let secs = epoch_ms.div_euclid(1000);
    let millis = epoch_ms.rem_euclid(1000);
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = tod / 3600;
    let minute = (tod % 3600) / 60;
    let second = tod % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// 公历换算（Howard Hinnant 的 `civil_from_days`，全整数、无分支依赖）。
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}
