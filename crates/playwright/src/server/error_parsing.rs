//! Classification of Playwright protocol error responses into `Error` values.
//!
//! Pure functions split out from the async connection machinery so they can be
//! unit-tested (and mutation-tested) without a live server.

use crate::error::Error;
use crate::server::connection::{ErrorPayload, ExpectErrorDetails};

/// Detects if an error message indicates a browser installation issue
fn is_browser_installation_error(message: &str) -> bool {
    message.contains("Looks like Playwright")
        || message.contains("Executable doesn't exist")
        || message.contains("not installed")
        || message.contains("Please run")
}

/// Extracts browser name from error message
fn extract_browser_name(message: &str) -> &str {
    // Check in priority order (specific to generic)
    if message.contains("chromium") {
        "chromium"
    } else if message.contains("firefox") {
        "firefox"
    } else if message.contains("webkit") {
        "webkit"
    } else {
        // If we can't detect the browser, use a generic message
        "browsers"
    }
}

pub(crate) fn parse_protocol_error(
    payload: ErrorPayload,
    details: Option<ExpectErrorDetails>,
) -> Error {
    // Auto-retrying assertions (`Frame.expect`) report a mismatch or timeout via
    // structured `errorDetails`. The 1.61 driver attaches `errorDetails` to
    // EVERY error thrown from `expect`, though — including infrastructure
    // failures such as the target closing mid-retry — for which it is an empty
    // `{}`. Only a genuine assertion result populates a field, so classify on
    // content: an empty details object is a real error and falls through to the
    // generic handling below.
    if let Some(details) = details
        && (details.timed_out.is_some()
            || details.custom_error_message.is_some()
            || details.received.is_some())
    {
        let message = details.custom_error_message.unwrap_or(payload.message);
        return if details.timed_out.unwrap_or(false) {
            Error::AssertionTimeout(message)
        } else {
            Error::AssertionFailed(message)
        };
    }

    // Detect browser installation errors
    // Playwright server sends errors with messages like:
    // "Looks like Playwright Test or Playwright was just installed or updated."
    // or "browserType.launch: Executable doesn't exist at /path/to/chromium"

    let message = &payload.message;

    // Check for browser installation errors
    if is_browser_installation_error(message) {
        let browser_name = extract_browser_name(message);

        return Error::BrowserNotInstalled {
            browser_name: browser_name.to_string(),
            message: message.clone(),
            playwright_version: crate::PLAYWRIGHT_VERSION.to_string(),
        };
    }

    // Default: return as protocol error
    Error::ProtocolError(format!(
        "{} \n {}",
        payload.message,
        payload.stack.unwrap_or_default()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(message: &str) -> ErrorPayload {
        ErrorPayload {
            message: message.to_string(),
            name: None,
            stack: None,
        }
    }

    fn details(
        timed_out: Option<bool>,
        custom: Option<&str>,
        received: Option<serde_json::Value>,
    ) -> ExpectErrorDetails {
        ExpectErrorDetails {
            timed_out,
            custom_error_message: custom.map(str::to_string),
            received,
        }
    }

    #[test]
    fn expect_details_timed_out_maps_to_assertion_timeout() {
        let err = parse_protocol_error(
            payload("timeout"),
            Some(details(Some(true), Some("nope"), None)),
        );
        assert!(matches!(err, Error::AssertionTimeout(msg) if msg == "nope"));
    }

    #[test]
    fn expect_details_timed_out_alone_maps_to_assertion_timeout() {
        // A timeout before any intermediate result: `timedOut` set, no custom
        // message, no `received`. Must still classify as a timeout.
        let err = parse_protocol_error(payload("timed out"), Some(details(Some(true), None, None)));
        assert!(matches!(err, Error::AssertionTimeout(msg) if msg == "timed out"));
    }

    #[test]
    fn expect_details_without_timeout_maps_to_assertion_failed() {
        let err = parse_protocol_error(
            payload("base"),
            Some(details(Some(false), Some("nope"), None)),
        );
        assert!(matches!(err, Error::AssertionFailed(msg) if msg == "nope"));
    }

    #[test]
    fn expect_details_falls_back_to_payload_message_when_no_custom() {
        // A genuine mismatch with no custom message still carries `received`;
        // the message then falls back to the payload's own text.
        let err = parse_protocol_error(
            payload("base message"),
            Some(details(
                None,
                None,
                Some(serde_json::json!({ "value": "x" })),
            )),
        );
        assert!(matches!(err, Error::AssertionFailed(msg) if msg == "base message"));
    }

    #[test]
    fn expect_details_with_only_received_maps_to_assertion_failed() {
        // `received` present but no timeout / custom message → assertion failure,
        // not a timeout.
        let err = parse_protocol_error(
            payload("mismatch"),
            Some(details(None, None, Some(serde_json::json!({ "value": 1 })))),
        );
        assert!(matches!(err, Error::AssertionFailed(_)));
    }

    #[test]
    fn no_details_is_a_plain_protocol_error() {
        let err = parse_protocol_error(payload("boom"), None);
        assert!(matches!(err, Error::ProtocolError(_)));
    }

    #[test]
    fn empty_expect_details_is_a_protocol_error_not_an_assertion() {
        let empty = ExpectErrorDetails {
            timed_out: None,
            custom_error_message: None,
            received: None,
        };
        let err = parse_protocol_error(
            payload("Target page, context or browser has been closed"),
            Some(empty),
        );
        assert!(matches!(err, Error::ProtocolError(_)));
    }

    #[test]
    fn no_details_still_detects_browser_install_errors() {
        let err = parse_protocol_error(payload("Executable doesn't exist for chromium"), None);
        assert!(matches!(err, Error::BrowserNotInstalled { .. }));
    }

    #[test]
    fn each_install_error_phrase_is_detected_independently() {
        // Each phrase alone must trip detection, so an `||` -> `&&` regression
        // (which would require two phrases at once) breaks at least one case.
        assert!(is_browser_installation_error(
            "Looks like Playwright was just installed"
        ));
        assert!(is_browser_installation_error(
            "Executable doesn't exist at /x"
        ));
        assert!(is_browser_installation_error(
            "the browser is not installed"
        ));
        assert!(is_browser_installation_error(
            "Please run playwright install"
        ));
        assert!(!is_browser_installation_error("an unrelated failure"));
    }

    #[test]
    fn browser_name_extracted_per_engine() {
        assert_eq!(extract_browser_name("launching firefox failed"), "firefox");
        assert_eq!(extract_browser_name("webkit missing"), "webkit");
        assert_eq!(extract_browser_name("chromium gone"), "chromium");
        assert_eq!(extract_browser_name("something generic"), "browsers");
    }
}
