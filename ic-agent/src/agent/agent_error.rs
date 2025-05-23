//! Errors that can occur when using the replica agent.

use crate::{agent::status::Status, RequestIdError};
use candid::Principal;
use ic_certification::Label;
use ic_transport_types::{InvalidRejectCodeError, RejectResponse};
use leb128::read;
use std::time::Duration;
use std::{
    fmt::{Debug, Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// An error that occurred when using the agent.
#[derive(Error, Debug)]
pub enum AgentError {
    /// The replica URL was invalid.
    #[error(r#"Invalid Replica URL: "{0}""#)]
    InvalidReplicaUrl(String),

    /// The request timed out.
    #[error("The request timed out.")]
    TimeoutWaitingForResponse(),

    /// An error occurred when signing with the identity.
    #[error("Identity had a signing error: {0}")]
    SigningError(String),

    /// The data fetched was invalid CBOR.
    #[error("Invalid CBOR data, could not deserialize: {0}")]
    InvalidCborData(#[from] serde_cbor::Error),

    /// There was an error calculating a request ID.
    #[error("Cannot calculate a RequestID: {0}")]
    CannotCalculateRequestId(#[from] RequestIdError),

    /// There was an error when de/serializing with Candid.
    #[error("Candid returned an error: {0}")]
    CandidError(Box<dyn Send + Sync + std::error::Error>),

    /// There was an error parsing a URL.
    #[error(r#"Cannot parse url: "{0}""#)]
    UrlParseError(#[from] url::ParseError),

    /// The HTTP method was invalid.
    #[error(r#"Invalid method: "{0}""#)]
    InvalidMethodError(#[from] http::method::InvalidMethod),

    /// The principal string was not a valid principal.
    #[error("Cannot parse Principal: {0}")]
    PrincipalError(#[from] crate::export::PrincipalError),

    /// The subnet rejected the message.
    #[error("The replica returned a rejection error: reject code {:?}, reject message {}, error code {:?}", .reject.reject_code, .reject.reject_message, .reject.error_code)]
    CertifiedReject {
        /// The rejection returned by the replica.
        reject: RejectResponse,
        /// The operation that was rejected. Not always available.
        operation: Option<Operation>,
    },

    /// The subnet may have rejected the message. This rejection cannot be verified as authentic.
    #[error("The replica returned a rejection error: reject code {:?}, reject message {}, error code {:?}", .reject.reject_code, .reject.reject_message, .reject.error_code)]
    UncertifiedReject {
        /// The rejection returned by the boundary node.
        reject: RejectResponse,
        /// The operation that was rejected. Not always available.
        operation: Option<Operation>,
    },

    /// The replica returned an HTTP error.
    #[error("The replica returned an HTTP Error: {0}")]
    HttpError(HttpErrorPayload),

    /// The status endpoint returned an invalid status.
    #[error("Status endpoint returned an invalid status.")]
    InvalidReplicaStatus,

    /// The call was marked done, but no reply was provided.
    #[error("Call was marked as done but we never saw the reply. Request ID: {0}")]
    RequestStatusDoneNoReply(String),

    /// A string error occurred in an external tool.
    #[error("A tool returned a string message error: {0}")]
    MessageError(String),

    /// There was an error reading a LEB128 value.
    #[error("Error reading LEB128 value: {0}")]
    Leb128ReadError(#[from] read::Error),

    /// A string was invalid UTF-8.
    #[error("Error in UTF-8 string: {0}")]
    Utf8ReadError(#[from] Utf8Error),

    /// The lookup path was absent in the certificate.
    #[error("The lookup path ({0:?}) is absent in the certificate.")]
    LookupPathAbsent(Vec<Label>),

    /// The lookup path was unknown in the certificate.
    #[error("The lookup path ({0:?}) is unknown in the certificate.")]
    LookupPathUnknown(Vec<Label>),

    /// The lookup path did not make sense for the certificate.
    #[error("The lookup path ({0:?}) does not make sense for the certificate.")]
    LookupPathError(Vec<Label>),

    /// The request status at the requested path was invalid.
    #[error("The request status ({1}) at path {0:?} is invalid.")]
    InvalidRequestStatus(Vec<Label>, String),

    /// The certificate verification for a `read_state` call failed.
    #[error("Certificate verification failed.")]
    CertificateVerificationFailed(),

    /// The signature verification for a query call failed.
    #[error("Query signature verification failed.")]
    QuerySignatureVerificationFailed,

    /// The certificate contained a delegation that does not include the `effective_canister_id` in the `canister_ranges` field.
    #[error("Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?")]
    CertificateNotAuthorized(),

    /// The certificate was older than allowed by the `ingress_expiry`.
    #[error("Certificate is stale (over {0:?}). Is the computer's clock synchronized?")]
    CertificateOutdated(Duration),

    /// The certificate contained more than one delegation.
    #[error("The certificate contained more than one delegation")]
    CertificateHasTooManyDelegations,

    /// The query response did not contain any node signatures.
    #[error("Query response did not contain any node signatures")]
    MissingSignature,

    /// The query response contained a malformed signature.
    #[error("Query response contained a malformed signature")]
    MalformedSignature,

    /// The read-state response contained a malformed public key.
    #[error("Read state response contained a malformed public key")]
    MalformedPublicKey,

    /// The query response contained more node signatures than the subnet has nodes.
    #[error("Query response contained too many signatures ({had}, exceeding the subnet's total nodes: {needed})")]
    TooManySignatures {
        /// The number of provided signatures.
        had: usize,
        /// The number of nodes on the subnet.
        needed: usize,
    },

    /// There was a length mismatch between the expected and actual length of the BLS DER-encoded public key.
    #[error(
        r#"BLS DER-encoded public key must be ${expected} bytes long, but is {actual} bytes long."#
    )]
    DerKeyLengthMismatch {
        /// The expected length of the key.
        expected: usize,
        /// The actual length of the key.
        actual: usize,
    },

    /// There was a mismatch between the expected and actual prefix of the BLS DER-encoded public key.
    #[error("BLS DER-encoded public key is invalid. Expected the following prefix: ${expected:?}, but got ${actual:?}")]
    DerPrefixMismatch {
        /// The expected key prefix.
        expected: Vec<u8>,
        /// The actual key prefix.
        actual: Vec<u8>,
    },

    /// The status response did not contain a root key.
    #[error("The status response did not contain a root key.  Status: {0}")]
    NoRootKeyInStatus(Status),

    /// The invocation to the wallet call forward method failed with an error.
    #[error("The invocation to the wallet call forward method failed with the error: {0}")]
    WalletCallFailed(String),

    /// The wallet operation failed.
    #[error("The  wallet operation failed: {0}")]
    WalletError(String),

    /// The wallet canister must be upgraded. See [`dfx wallet upgrade`](https://internetcomputer.org/docs/current/references/cli-reference/dfx-wallet)
    #[error("The wallet canister must be upgraded: {0}")]
    WalletUpgradeRequired(String),

    /// The response size exceeded the provided limit.
    #[error("Response size exceeded limit.")]
    ResponseSizeExceededLimit(),

    /// An unknown error occurred during communication with the replica.
    #[error("An error happened during communication with the replica: {0}")]
    TransportError(#[from] reqwest::Error),

    /// There was a mismatch between the expected and actual CBOR data during inspection.
    #[error("There is a mismatch between the CBOR encoded call and the arguments: field {field}, value in argument is {value_arg}, value in CBOR is {value_cbor}")]
    CallDataMismatch {
        /// The field that was mismatched.
        field: String,
        /// The value that was expected to be in the CBOR.
        value_arg: String,
        /// The value that was actually in the CBOR.
        value_cbor: String,
    },

    /// The rejected call had an invalid reject code (valid range 1..5).
    #[error(transparent)]
    InvalidRejectCode(#[from] InvalidRejectCodeError),

    /// Route provider failed to generate a url for some reason.
    #[error("Route provider failed to generate url: {0}")]
    RouteProviderError(String),

    /// Invalid HTTP response.
    #[error("Invalid HTTP response: {0}")]
    InvalidHttpResponse(String),
}

impl PartialEq for AgentError {
    fn eq(&self, other: &Self) -> bool {
        // Verify the debug string is the same. Some of the subtypes of this error
        // don't implement Eq or PartialEq, so we cannot rely on derive.
        format!("{self:?}") == format!("{other:?}")
    }
}

impl From<candid::Error> for AgentError {
    fn from(e: candid::Error) -> AgentError {
        AgentError::CandidError(e.into())
    }
}

/// A HTTP error from the replica.
pub struct HttpErrorPayload {
    /// The HTTP status code.
    pub status: u16,
    /// The MIME type of `content`.
    pub content_type: Option<String>,
    /// The body of the error.
    pub content: Vec<u8>,
}

impl HttpErrorPayload {
    fn fmt_human_readable(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        // No matter content_type is TEXT or not,
        // always try to parse it as a String.
        // When fail, print the raw byte array
        f.write_fmt(format_args!(
            "Http Error: status {}, content type {:?}, content: {}",
            http::StatusCode::from_u16(self.status)
                .map_or_else(|_| format!("{}", self.status), |code| format!("{code}")),
            self.content_type.clone().unwrap_or_default(),
            String::from_utf8(self.content.clone()).unwrap_or_else(|_| format!(
                "(unable to decode content as UTF-8: {:?})",
                self.content
            ))
        ))?;
        Ok(())
    }
}

impl Debug for HttpErrorPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.fmt_human_readable(f)
    }
}

impl Display for HttpErrorPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.fmt_human_readable(f)
    }
}

/// An operation that can result in a reject.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operation {
    /// A call to a canister method.
    Call {
        /// The canister whose method was called.
        canister: Principal,
        /// The name of the method.
        method: String,
    },
    /// A read of the state tree, in the context of a canister. This will *not* be returned for request polling.
    ReadState {
        /// The requested paths within the state tree.
        paths: Vec<Vec<String>>,
        /// The canister the read request was made in the context of.
        canister: Principal,
    },
    /// A read of the state tree, in the context of a subnet.
    ReadSubnetState {
        /// The requested paths within the state tree.
        paths: Vec<Vec<String>>,
        /// The subnet the read request was made in the context of.
        subnet: Principal,
    },
}

#[cfg(test)]
mod tests {
    use super::HttpErrorPayload;
    use crate::AgentError;

    #[test]
    fn content_type_none_valid_utf8() {
        let payload = HttpErrorPayload {
            status: 420,
            content_type: None,
            content: vec![104, 101, 108, 108, 111],
        };

        assert_eq!(
            format!("{}", AgentError::HttpError(payload)),
            r#"The replica returned an HTTP Error: Http Error: status 420 <unknown status code>, content type "", content: hello"#,
        );
    }

    #[test]
    fn content_type_none_invalid_utf8() {
        let payload = HttpErrorPayload {
            status: 420,
            content_type: None,
            content: vec![195, 40],
        };

        assert_eq!(
            format!("{}", AgentError::HttpError(payload)),
            r#"The replica returned an HTTP Error: Http Error: status 420 <unknown status code>, content type "", content: (unable to decode content as UTF-8: [195, 40])"#,
        );
    }

    #[test]
    fn formats_text_plain() {
        let payload = HttpErrorPayload {
            status: 420,
            content_type: Some("text/plain".to_string()),
            content: vec![104, 101, 108, 108, 111],
        };

        assert_eq!(
            format!("{}", AgentError::HttpError(payload)),
            r#"The replica returned an HTTP Error: Http Error: status 420 <unknown status code>, content type "text/plain", content: hello"#,
        );
    }

    #[test]
    fn formats_text_plain_charset_utf8() {
        let payload = HttpErrorPayload {
            status: 420,
            content_type: Some("text/plain; charset=utf-8".to_string()),
            content: vec![104, 101, 108, 108, 111],
        };

        assert_eq!(
            format!("{}", AgentError::HttpError(payload)),
            r#"The replica returned an HTTP Error: Http Error: status 420 <unknown status code>, content type "text/plain; charset=utf-8", content: hello"#,
        );
    }

    #[test]
    fn formats_text_html() {
        let payload = HttpErrorPayload {
            status: 420,
            content_type: Some("text/html".to_string()),
            content: vec![119, 111, 114, 108, 100],
        };

        assert_eq!(
            format!("{}", AgentError::HttpError(payload)),
            r#"The replica returned an HTTP Error: Http Error: status 420 <unknown status code>, content type "text/html", content: world"#,
        );
    }
}
