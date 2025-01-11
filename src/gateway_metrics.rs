use metrics::{counter, histogram};

const REQUESTS_TOTAL: &str = "gateway.requests.total";
const RESPONSES_TOTAL: &str = "gateway.responses.total";
const FILTERS_DURATION_SECONDS: &str = "gateway.filters.duration_seconds";

/// Increments the "gateway.requests.total" counter by the given amount.
pub fn inc_requests_total() {
    // The `counter!` macro returns a `metrics::Counter` handle each time.
    // We increment by the specified amount here.
    counter!(REQUESTS_TOTAL).increment(1);
}

/// Increments the "gateway.responses.total" counter by the given amount.
pub fn inc_responses_total() {
    counter!(RESPONSES_TOTAL).increment(1);
}

/// Records a duration in seconds to "gateway.filters.duration_seconds".
pub fn record_filter_duration(duration: f64) {
    histogram!(FILTERS_DURATION_SECONDS).record(duration);
}
