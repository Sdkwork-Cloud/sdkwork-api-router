use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{MatchedPath, State};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static TRACING_INIT: OnceLock<()> = OnceLock::new();

pub fn service_name(name: &str) -> &str {
    name
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestId(String);

impl RequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct HttpMetricsRegistry {
    service: Arc<str>,
    metrics: Arc<Mutex<BTreeMap<HttpMetricKey, HttpMetricValue>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HttpMetricKey {
    method: String,
    route: String,
    status: u16,
}

#[derive(Debug, Clone, Default)]
struct HttpMetricValue {
    count: u64,
    duration_ms_sum: u64,
}

impl HttpMetricsRegistry {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into().into(),
            metrics: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn record(&self, method: &str, route: &str, status: u16, duration_ms: u64) {
        let key = HttpMetricKey {
            method: method.to_owned(),
            route: route.to_owned(),
            status,
        };

        let mut metrics = match self.metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = metrics.entry(key).or_default();
        entry.count += 1;
        entry.duration_ms_sum += duration_ms;
    }

    pub fn render_prometheus(&self) -> String {
        let metrics = match self.metrics.lock() {
            Ok(metrics) => metrics,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut output = String::new();
        output.push_str("# HELP sdkwork_service_info Static service identity metric\n");
        output.push_str("# TYPE sdkwork_service_info gauge\n");
        output.push_str(&format!(
            "sdkwork_service_info{{service=\"{}\"}} 1\n",
            escape_label(self.service())
        ));

        output.push_str("# HELP sdkwork_http_requests_total Total HTTP requests observed\n");
        output.push_str("# TYPE sdkwork_http_requests_total counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_requests_total{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.count
            ));
        }

        output.push_str(
            "# HELP sdkwork_http_request_duration_ms_sum Cumulative request duration in milliseconds\n",
        );
        output.push_str("# TYPE sdkwork_http_request_duration_ms_sum counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_sum{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.duration_ms_sum
            ));
        }

        output.push_str(
            "# HELP sdkwork_http_request_duration_ms_count Request count paired with duration summaries\n",
        );
        output.push_str("# TYPE sdkwork_http_request_duration_ms_count counter\n");
        for (key, value) in metrics.iter() {
            output.push_str(&format!(
                "sdkwork_http_request_duration_ms_count{{service=\"{}\",method=\"{}\",route=\"{}\",status=\"{}\"}} {}\n",
                escape_label(self.service()),
                escape_label(&key.method),
                escape_label(&key.route),
                key.status,
                value.count
            ));
        }

        output
    }
}

pub async fn observe_http_metrics(
    State(registry): State<Arc<HttpMetricsRegistry>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let started_at = Instant::now();
    let response = next.run(request).await;
    let duration_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    registry.record(&method, &route, status, duration_ms);
    response
}

pub async fn observe_http_tracing(
    State(service): State<Arc<str>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let request_id = resolved_request_id(&request);
    request
        .extensions_mut()
        .insert(RequestId::new(request_id.clone()));
    let started_at = Instant::now();
    let span = tracing::info_span!(
        "http_request",
        service = %service,
        request_id = %request_id,
        method = %method,
        route = %route
    );
    let mut response = next.run(request).instrument(span.clone()).await;
    let duration_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
    tracing::info!(parent: &span, status, duration_ms, "completed request");
    response
}

pub fn init_tracing(service: &str) {
    TRACING_INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .compact()
            .with_target(false)
            .with_max_level(tracing::Level::INFO)
            .try_init();
        tracing::info!(service = service, "tracing initialized");
    });
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn resolved_request_id(request: &Request<axum::body::Body>) -> String {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(generate_request_id)
}

fn generate_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let sequence = REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("sdkw-{millis:x}-{sequence:x}")
}
