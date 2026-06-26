//! 外向き HTTP の共有ヘルパー。LLM API 通信とモデルのダウンロードで使う。
//!
//! 企業ネットワークを想定し、プロキシと独自CAを環境変数から読む。
//!
//! - プロキシ: ureq の既定設定が `ALL_PROXY` / `HTTP_PROXY` / `HTTPS_PROXY`
//!   （および `NO_PROXY`）を読む。ここでは既定の `config_builder()` を使うため
//!   追加設定は不要。
//! - 独自CA: `ATOM_CA_BUNDLE` / `SSL_CERT_FILE` / `CURL_CA_BUNDLE` のいずれかが
//!   指す PEM バンドルをルート証明書として使う（curl / OpenSSL と同じく、既定の
//!   ルートを置き換える）。TLS 傍受プロキシ環境での利用を想定。

use std::time::Duration;

/// CA バンドルのパスを読む環境変数（優先順）。
const CA_ENV_VARS: [&str; 3] = ["ATOM_CA_BUNDLE", "SSL_CERT_FILE", "CURL_CA_BUNDLE"];

/// 共有設定の ureq Agent を作る。プロキシは環境変数から自動で読まれる。
/// 独自CA が環境変数で与えられていればルート証明書として適用する。
pub fn agent(timeout: Option<Duration>) -> ureq::Agent {
    let mut builder = ureq::Agent::config_builder();
    if let Some(timeout) = timeout {
        builder = builder.timeout_global(Some(timeout));
    }
    if let Some(tls) = custom_ca_tls_config() {
        builder = builder.tls_config(tls);
    }
    builder.build().new_agent()
}

/// 環境変数で独自CAバンドルが指定されていれば、それを使う TLS 設定を返す。
fn custom_ca_tls_config() -> Option<ureq::tls::TlsConfig> {
    let path = CA_ENV_VARS.iter().find_map(std::env::var_os)?;
    let pem = std::fs::read(&path).ok()?;
    let certs: Vec<ureq::tls::Certificate<'static>> = ureq::tls::parse_pem(&pem)
        .filter_map(Result::ok)
        .filter_map(|item| match item {
            ureq::tls::PemItem::Certificate(cert) => Some(cert),
            _ => None,
        })
        .collect();
    if certs.is_empty() {
        return None;
    }
    let roots = ureq::tls::RootCerts::from(certs);
    Some(ureq::tls::TlsConfig::builder().root_certs(roots).build())
}
