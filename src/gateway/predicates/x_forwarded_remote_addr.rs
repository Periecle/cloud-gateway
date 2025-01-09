use std::collections::HashSet;
use std::net::IpAddr;

use hyper::Request;

use super::Evaluable;

/// Ultra-optimized XForwardedRemoteAddrPredicate using HashSet for membership tests.
#[derive(Debug, Clone)]
pub struct XForwardedRemoteAddrPredicate {
    /// HashSet for O(1) average membership checks
    pub addrs: HashSet<IpAddr>,
}

impl<T> Evaluable<T> for XForwardedRemoteAddrPredicate {
    #[inline(always)]
    fn evaluate(&self, request: &Request<T>) -> bool {
        // Quickly return if header is missing or invalid
        let header_value = match request.headers().get("x-forwarded-for") {
            Some(hv) => hv,
            None => return false,
        };
        let header_str = match header_value.to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Convert to bytes for manual parsing
        let bytes = header_str.as_bytes();
        let mut start = 0;
        let n = bytes.len();

        while start < n {
            // Skip leading spaces
            while start < n && (bytes[start] == b' ' || bytes[start] == b'\t') {
                start += 1;
            }

            // Find comma or end-of-string
            let mut end = start;
            while end < n && bytes[end] != b',' {
                end += 1;
            }

            // Extract substring without allocating
            let ip_bytes = &bytes[start..end];

            // Skip trailing whitespace
            let mut ip_end = ip_bytes.len();
            while ip_end > 0 && (ip_bytes[ip_end - 1] == b' ' || ip_bytes[ip_end - 1] == b'\t') {
                ip_end -= 1;
            }

            // Convert to &str (unsafe because we assume valid ASCII/UTF-8 in headers)
            let raw_ip = unsafe { std::str::from_utf8_unchecked(&ip_bytes[..ip_end]) };

            // Parse and check membership
            if let Ok(ip) = raw_ip.parse::<IpAddr>() {
                if self.addrs.contains(&ip) {
                    return true;
                }
            }

            // Move start to character after comma
            start = end + 1;
        }

        false
    }
}
