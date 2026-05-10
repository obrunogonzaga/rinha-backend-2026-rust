use serde::Deserialize;

const MAX_AMOUNT: f32 = 10_000.0;
const MAX_INSTALLMENTS: f32 = 12.0;
const AMOUNT_VS_AVG_RATIO: f32 = 10.0;
const MAX_MINUTES: f32 = 1440.0;
const MAX_KM: f32 = 1000.0;
const MAX_TX_COUNT_24H: f32 = 20.0;
const MAX_MERCHANT_AVG_AMOUNT: f32 = 10_000.0;

const MCC_DEFAULT: f32 = 0.5;
const MCC_TABLE: &[(&str, f32)] = &[
    ("4511", 0.35),
    ("5311", 0.25),
    ("5411", 0.15),
    ("5812", 0.30),
    ("5912", 0.20),
    ("5944", 0.45),
    ("5999", 0.50),
    ("7801", 0.80),
    ("7802", 0.75),
    ("7995", 0.85),
];

const SENTINEL_NULL: f32 = -1.0;
const SECONDS_PER_MINUTE: i64 = 60;

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub transaction: Transaction,
    pub customer: Customer,
    pub merchant: Merchant,
    pub terminal: Terminal,
    pub last_transaction: Option<LastTransaction>,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub amount: f32,
    pub installments: u32,
    pub requested_at: String,
}

#[derive(Deserialize, Debug)]
pub struct Customer {
    pub avg_amount: f32,
    pub tx_count_24h: u32,
    pub known_merchants: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Merchant {
    pub id: String,
    pub mcc: String,
    pub avg_amount: f32,
}

#[derive(Deserialize, Debug)]
pub struct Terminal {
    pub is_online: bool,
    pub card_present: bool,
    pub km_from_home: f32,
}

#[derive(Deserialize, Debug)]
pub struct LastTransaction {
    pub timestamp: String,
    pub km_from_current: f32,
}

pub fn vectorize(p: &Payload) -> [f32; 14] {
    let ts = parse_iso_utc(&p.transaction.requested_at);

    let (minutes_since, km_from_last) = match &p.last_transaction {
        None => (SENTINEL_NULL, SENTINEL_NULL),
        Some(last) => {
            let prev = parse_iso_utc(&last.timestamp);
            let diff_min = (ts.epoch_seconds - prev.epoch_seconds) / SECONDS_PER_MINUTE;
            (
                clamp01(diff_min as f32 / MAX_MINUTES),
                clamp01(last.km_from_current / MAX_KM),
            )
        }
    };

    [
        clamp01(p.transaction.amount / MAX_AMOUNT),
        clamp01(p.transaction.installments as f32 / MAX_INSTALLMENTS),
        clamp01((p.transaction.amount / p.customer.avg_amount) / AMOUNT_VS_AVG_RATIO),
        ts.hour as f32 / 23.0,
        ts.weekday_mon0 as f32 / 6.0,
        minutes_since,
        km_from_last,
        clamp01(p.terminal.km_from_home / MAX_KM),
        clamp01(p.customer.tx_count_24h as f32 / MAX_TX_COUNT_24H),
        bool_to_f32(p.terminal.is_online),
        bool_to_f32(p.terminal.card_present),
        if is_known(&p.merchant.id, &p.customer.known_merchants) {
            0.0
        } else {
            1.0
        },
        mcc_risk(&p.merchant.mcc),
        clamp01(p.merchant.avg_amount / MAX_MERCHANT_AVG_AMOUNT),
    ]
}

fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

fn bool_to_f32(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

fn is_known(id: &str, known: &[String]) -> bool {
    known.iter().any(|m| m == id)
}

fn mcc_risk(mcc: &str) -> f32 {
    for (k, v) in MCC_TABLE {
        if *k == mcc {
            return *v;
        }
    }
    MCC_DEFAULT
}

struct Timestamp {
    hour: u32,
    weekday_mon0: u32,
    epoch_seconds: i64,
}

fn parse_iso_utc(s: &str) -> Timestamp {
    let b = s.as_bytes();
    debug_assert!(
        b.len() == 20 && b[19] == b'Z',
        "expected YYYY-MM-DDTHH:MM:SSZ"
    );
    let year = parse_u32(&b[0..4]);
    let month = parse_u32(&b[5..7]);
    let day = parse_u32(&b[8..10]);
    let hour = parse_u32(&b[11..13]);
    let minute = parse_u32(&b[14..16]);
    let second = parse_u32(&b[17..19]);
    Timestamp {
        hour,
        weekday_mon0: weekday_mon0(year, month, day),
        epoch_seconds: epoch_seconds(year, month, day, hour, minute, second),
    }
}

fn parse_u32(b: &[u8]) -> u32 {
    let mut acc: u32 = 0;
    for byte in b {
        acc = acc * 10 + (byte - b'0') as u32;
    }
    acc
}

fn weekday_mon0(year: u32, month: u32, day: u32) -> u32 {
    const T: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let sun0 = (y + y / 4 - y / 100 + y / 400 + T[(month - 1) as usize] + day) % 7;
    (sun0 + 6) % 7
}

fn epoch_seconds(year: u32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> i64 {
    let days = days_from_civil(year as i64, month, day);
    days * 86_400 + (hour * 3600 + minute * 60 + second) as i64
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m_adj = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * m_adj + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGIT_PAYLOAD: &str = r#"{
        "id": "tx-1329056812",
        "transaction": { "amount": 41.12, "installments": 2, "requested_at": "2026-03-11T18:45:53Z" },
        "customer": { "avg_amount": 82.24, "tx_count_24h": 3, "known_merchants": ["MERC-003", "MERC-016"] },
        "merchant": { "id": "MERC-016", "mcc": "5411", "avg_amount": 60.25 },
        "terminal": { "is_online": false, "card_present": true, "km_from_home": 29.2331036248 },
        "last_transaction": null
    }"#;

    const FRAUD_PAYLOAD: &str = r#"{
        "id": "tx-3330991687",
        "transaction": { "amount": 9505.97, "installments": 10, "requested_at": "2026-03-14T05:15:12Z" },
        "customer": { "avg_amount": 81.28, "tx_count_24h": 20, "known_merchants": ["MERC-008", "MERC-007", "MERC-005"] },
        "merchant": { "id": "MERC-068", "mcc": "7802", "avg_amount": 54.86 },
        "terminal": { "is_online": false, "card_present": true, "km_from_home": 952.2745933273 },
        "last_transaction": null
    }"#;

    fn approx_eq_4dp(actual: &[f32; 14], expected: &[f32; 14]) {
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            let rounded = (a * 10_000.0).round() / 10_000.0;
            assert!(
                (rounded - e).abs() <= 1e-4,
                "dim {i}: actual={a} (rounded={rounded}), expected={e}"
            );
        }
    }

    #[test]
    fn vectorize_legitimate_example_matches_doc() {
        let p: Payload = serde_json::from_str(LEGIT_PAYLOAD).unwrap();
        let v = vectorize(&p);
        let expected = [
            0.0041, 0.1667, 0.05, 0.7826, 0.3333, -1.0, -1.0, 0.0292, 0.15, 0.0, 1.0, 0.0, 0.15,
            0.006,
        ];
        approx_eq_4dp(&v, &expected);
    }

    #[test]
    fn vectorize_fraud_example_matches_doc() {
        let p: Payload = serde_json::from_str(FRAUD_PAYLOAD).unwrap();
        let v = vectorize(&p);
        let expected = [
            0.9506, 0.8333, 1.0, 0.2174, 0.8333, -1.0, -1.0, 0.9523, 1.0, 0.0, 1.0, 1.0, 0.75,
            0.0055,
        ];
        approx_eq_4dp(&v, &expected);
    }

    #[test]
    fn vectorize_last_transaction_null_writes_minus_one_at_5_and_6() {
        let p: Payload = serde_json::from_str(LEGIT_PAYLOAD).unwrap();
        let v = vectorize(&p);
        assert_eq!(v[5], -1.0);
        assert_eq!(v[6], -1.0);
    }

    #[test]
    fn vectorize_with_last_transaction_computes_minutes_and_km() {
        let payload = r#"{
            "id": "tx-x",
            "transaction": { "amount": 100.0, "installments": 1, "requested_at": "2026-03-11T20:23:35Z" },
            "customer": { "avg_amount": 100.0, "tx_count_24h": 1, "known_merchants": [] },
            "merchant": { "id": "MERC-001", "mcc": "5912", "avg_amount": 100.0 },
            "terminal": { "is_online": true, "card_present": false, "km_from_home": 100.0 },
            "last_transaction": { "timestamp": "2026-03-11T14:58:35Z", "km_from_current": 18.8626479774 }
        }"#;
        let p: Payload = serde_json::from_str(payload).unwrap();
        let v = vectorize(&p);
        let expected_minutes = 325.0_f32 / 1440.0;
        let expected_km = 18.862_648_f32 / 1000.0;
        assert!((v[5] - expected_minutes).abs() < 1e-6, "v[5]={}", v[5]);
        assert!((v[6] - expected_km).abs() < 1e-6, "v[6]={}", v[6]);
    }

    #[test]
    fn clamp_caps_amount_above_max() {
        let payload = r#"{
            "id": "tx-x",
            "transaction": { "amount": 50000.0, "installments": 24, "requested_at": "2026-03-11T18:45:53Z" },
            "customer": { "avg_amount": 1.0, "tx_count_24h": 100, "known_merchants": [] },
            "merchant": { "id": "MERC-X", "mcc": "5411", "avg_amount": 99999.0 },
            "terminal": { "is_online": true, "card_present": true, "km_from_home": 99999.0 },
            "last_transaction": null
        }"#;
        let p: Payload = serde_json::from_str(payload).unwrap();
        let v = vectorize(&p);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 1.0);
        assert_eq!(v[2], 1.0);
        assert_eq!(v[7], 1.0);
        assert_eq!(v[8], 1.0);
        assert_eq!(v[13], 1.0);
    }

    #[test]
    fn mcc_risk_unknown_returns_default() {
        assert_eq!(mcc_risk("9999"), 0.5);
        assert_eq!(mcc_risk(""), 0.5);
    }

    #[test]
    fn mcc_risk_known_returns_table_value() {
        assert_eq!(mcc_risk("7995"), 0.85);
        assert_eq!(mcc_risk("5411"), 0.15);
    }

    #[test]
    fn unknown_merchant_uses_set_membership_with_duplicates() {
        let known = vec!["MERC-001".to_string(), "MERC-001".to_string()];
        assert!(is_known("MERC-001", &known));
        assert!(!is_known("MERC-999", &known));
    }

    #[test]
    fn weekday_matches_known_dates() {
        assert_eq!(weekday_mon0(2026, 3, 11), 2); // Wed
        assert_eq!(weekday_mon0(2026, 3, 14), 5); // Sat
        assert_eq!(weekday_mon0(2026, 3, 8), 6); // Sun
        assert_eq!(weekday_mon0(2026, 3, 9), 0); // Mon
    }

    #[test]
    fn parse_u32_handles_zero_padded_fields() {
        assert_eq!(parse_u32(b"00"), 0);
        assert_eq!(parse_u32(b"05"), 5);
        assert_eq!(parse_u32(b"23"), 23);
        assert_eq!(parse_u32(b"2026"), 2026);
    }
}
