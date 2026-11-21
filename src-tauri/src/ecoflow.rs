//! EcoFlow IoT Open Platform client for STREAM (HTTP, HMAC-SHA256 signed).
//!
//! Source switching uses the backup reserve (`cfgBackupReverseSoc`) in Self-powered mode:
//! a low reserve lets the battery cover the house, a reserve at the current charge holds it.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context};
use hmac::{Hmac, Mac};
use parking_lot::Mutex;
use serde::Serialize;
use serde_json::{json, Map, Value};
use sha2::Sha256;

use crate::settings::Settings;

/// Live readings of the STREAM system (main device).
#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub soc: f64,
    pub reserve: i64,
    pub min_dsg: i64,
    pub max_chg: i64,
    pub battery_w: f64,
    pub load_w: f64,
    pub grid_w: f64,
    pub pv_w: f64,
    pub self_powered: bool,
    pub ai_mode: bool,
    pub feed_in: bool,
    /// «battery» | «grid» | «ai»
    pub source: &'static str,
    pub read_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Battery,
    Grid,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Battery => "від станції",
            Source::Grid => "від мережі",
        }
    }
}

pub struct EcoFlow {
    http: reqwest::Client,
    host: String,
    ak: String,
    sk: String,
    sn: String,
    main_sn: Mutex<Option<String>>,
}

impl EcoFlow {
    pub fn new(s: &Settings) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(concat!("LightOFF/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            http,
            host: s.api_host.clone(),
            ak: s.access_key.clone(),
            sk: s.secret_key.clone(),
            sn: s.sn.clone(),
            main_sn: Mutex::new(None),
        })
    }

    /// Signed request. GET params go to the query, PUT params to a JSON body; both are signed.
    async fn call(&self, method: reqwest::Method, path: &str, params: &Value) -> anyhow::Result<Value> {
        let nonce = nonce();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string();
        let sign = sign(&self.sk, &self.ak, &nonce, &ts, params)?;
        let url = format!("{}{}", self.host, path);

        let mut req = self
            .http
            .request(method.clone(), &url)
            .header("accessKey", &self.ak)
            .header("nonce", &nonce)
            .header("timestamp", &ts)
            .header("sign", sign);
        if method == reqwest::Method::GET {
            // No Content-Type on GET: with it the server ignores the query and rejects the signature (8521).
            let query: Vec<(String, String)> = flatten(params).into_iter().collect();
            req = req.query(&query);
        } else {
            req = req.header("Content-Type", "application/json;charset=UTF-8").body(params.to_string());
        }

        let resp = req.send().await.context("немає зв'язку з EcoFlow")?;
        let status = resp.status();
        let body: Value = resp.json().await.with_context(|| format!("EcoFlow: HTTP {status}"))?;
        let code = match body.get("code") {
            Some(Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => String::new(),
        };
        if code != "0" {
            let msg = body.get("message").and_then(Value::as_str).unwrap_or("невідома помилка");
            bail!("EcoFlow {code}: {msg}");
        }
        Ok(body.get("data").cloned().unwrap_or(Value::Null))
    }

    /// Serial of the system's main device; every command goes there.
    pub async fn main_sn(&self) -> anyhow::Result<String> {
        if let Some(sn) = self.main_sn.lock().clone() {
            return Ok(sn);
        }
        let data = self
            .call(reqwest::Method::GET, "/iot-open/sign/device/system/main/sn", &json!({ "sn": self.sn }))
            .await;
        let sn = match data {
            Ok(d) => d.get("sn").and_then(Value::as_str).unwrap_or(&self.sn).to_string(),
            Err(e) => {
                tracing::warn!(error = %e, "main sn lookup failed, using the configured SN");
                self.sn.clone()
            }
        };
        *self.main_sn.lock() = Some(sn.clone());
        Ok(sn)
    }

    pub async fn status(&self) -> anyhow::Result<Status> {
        let sn = self.main_sn().await?;
        let q = self.call(reqwest::Method::GET, "/iot-open/sign/device/quota/all", &json!({ "sn": sn })).await?;
        let f = |k: &str| q.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        let b = |k: &str| q.get(k).and_then(Value::as_bool).unwrap_or(false);
        if q.get("cmsBattSoc").is_none() {
            bail!("EcoFlow не повернув стан батареї (пристрій офлайн?)");
        }
        let soc = f("cmsBattSoc");
        let reserve = f("backupReverseSoc").round() as i64;
        let self_powered = b("energyStrategyOperateMode.operateSelfPoweredOpen");
        let ai_mode = b("energyStrategyOperateMode.operateIntelligentScheduleModeOpen");
        let source = if !self_powered {
            "ai"
        } else if reserve as f64 >= soc.floor() {
            "grid"
        } else {
            "battery"
        };
        Ok(Status {
            soc,
            reserve,
            min_dsg: f("cmsMinDsgSoc").round() as i64,
            max_chg: q.get("cmsMaxChgSoc").and_then(Value::as_f64).unwrap_or(100.0).round() as i64,
            battery_w: f("powGetBpCms"),
            load_w: f("powGetSysLoad"),
            grid_w: f("powGetSysGrid"),
            pv_w: f("powGetPvSum"),
            self_powered,
            ai_mode,
            feed_in: q.get("feedGridMode").and_then(Value::as_i64) == Some(2),
            source,
            read_at: chrono::Local::now().format("%H:%M:%S").to_string(),
        })
    }

    async fn set(&self, params: Value) -> anyhow::Result<()> {
        let sn = self.main_sn().await?;
        let body = json!({
            "sn": sn,
            "cmdId": 17,
            "cmdFunc": 254,
            "dirDest": 1,
            "dirSrc": 1,
            "dest": 2,
            "needAck": true,
            "params": params,
        });
        self.call(reqwest::Method::PUT, "/iot-open/sign/device/quota", &body).await.map(|_| ())
    }

    pub async fn set_reserve(&self, value: i64) -> anyhow::Result<()> {
        self.set(json!({ "cfgBackupReverseSoc": value })).await
    }

    pub async fn set_self_powered(&self) -> anyhow::Result<()> {
        self.set(json!({ "cfgEnergyStrategyOperateMode": { "operateSelfPoweredOpen": true } })).await
    }

    pub async fn set_ai_mode(&self) -> anyhow::Result<()> {
        self.set(json!({ "cfgEnergyStrategyOperateMode": { "operateIntelligentScheduleModeOpen": true } })).await
    }

    /// Switch the house source; returns the reserve that was written.
    pub async fn switch(&self, to: Source, s: &Settings) -> anyhow::Result<i64> {
        let st = self.status().await?;
        if !st.self_powered {
            self.set_self_powered().await.context("не вдалося увімкнути Self-powered")?;
        }
        let value = reserve_for(to, &st, s);
        self.set_reserve(value).await?;
        Ok(value)
    }
}

/// Reserve for a source, kept inside what the unit accepts (≥ discharge limit + 3, ≤ charge limit).
pub fn reserve_for(to: Source, st: &Status, s: &Settings) -> i64 {
    let lo = st.min_dsg + 3;
    let hi = st.max_chg.max(lo);
    let want = match to {
        Source::Battery => s.battery_reserve.map(i64::from).unwrap_or(lo),
        Source::Grid => s.grid_reserve.map(i64::from).unwrap_or(st.soc.ceil() as i64),
    };
    want.clamp(lo, hi)
}

fn nonce() -> String {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    format!("{}", 100_000 + n % 900_000)
}

/// `a.b=1&a.c[0]=x` pairs sorted by key, the format EcoFlow signs.
fn flatten(v: &Value) -> std::collections::BTreeMap<String, String> {
    fn walk(prefix: &str, v: &Value, out: &mut std::collections::BTreeMap<String, String>) {
        match v {
            Value::Object(m) => walk_map(prefix, m, out),
            Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    walk(&format!("{prefix}[{i}]"), x, out);
                }
            }
            Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            }
            Value::Null => {}
            other => {
                out.insert(prefix.to_string(), other.to_string());
            }
        }
    }
    fn walk_map(prefix: &str, m: &Map<String, Value>, out: &mut std::collections::BTreeMap<String, String>) {
        for (k, x) in m {
            let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
            walk(&key, x, out);
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk("", v, &mut out);
    out
}

fn sign(sk: &str, ak: &str, nonce: &str, ts: &str, params: &Value) -> anyhow::Result<String> {
    let mut parts: Vec<String> = flatten(params).into_iter().map(|(k, v)| format!("{k}={v}")).collect();
    parts.push(format!("accessKey={ak}"));
    parts.push(format!("nonce={nonce}"));
    parts.push(format!("timestamp={ts}"));
    let mut mac = Hmac::<Sha256>::new_from_slice(sk.as_bytes()).map_err(|e| anyhow!("ключ: {e}"))?;
    mac.update(parts.join("&").as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(soc: f64, min: i64, max: i64) -> Status {
        Status {
            soc,
            reserve: 0,
            min_dsg: min,
            max_chg: max,
            battery_w: 0.0,
            load_w: 0.0,
            grid_w: 0.0,
            pv_w: 0.0,
            self_powered: true,
            ai_mode: false,
            feed_in: true,
            source: "grid",
            read_at: String::new(),
        }
    }

    #[test]
    fn flatten_nested_and_bools() {
        let v = json!({"sn":"X","params":{"cfgEnergyStrategyOperateMode":{"operateSelfPoweredOpen":true}},"cmdId":17});
        let f = flatten(&v);
        assert_eq!(f["params.cfgEnergyStrategyOperateMode.operateSelfPoweredOpen"], "true");
        assert_eq!(f["cmdId"], "17");
        let keys: Vec<_> = f.keys().cloned().collect();
        assert_eq!(keys, vec!["cmdId", "params.cfgEnergyStrategyOperateMode.operateSelfPoweredOpen", "sn"]);
    }

    #[test]
    fn reserve_limits() {
        let s = Settings::default();
        let st = status(95.0, 10, 95);
        assert_eq!(reserve_for(Source::Battery, &st, &s), 13);
        assert_eq!(reserve_for(Source::Grid, &st, &s), 95);
        let st = status(42.3, 10, 95);
        assert_eq!(reserve_for(Source::Grid, &st, &s), 43);
        let s = Settings { battery_reserve: Some(5), grid_reserve: Some(100), ..Settings::default() };
        assert_eq!(reserve_for(Source::Battery, &st, &s), 13);
        assert_eq!(reserve_for(Source::Grid, &st, &s), 95);
    }
}
