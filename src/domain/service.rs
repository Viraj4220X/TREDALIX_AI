use serde::{Deserialize, Serialize};

pub const SERVICE_API_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiRequest<T> {
    pub api_version: u16,
    pub request_id: String,
    pub payload: T,
}

impl<T> ApiRequest<T> {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_envelope(self.api_version, &self.request_id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub api_version: u16,
    pub request_id: String,
    pub result: T,
}

impl<T> ApiResponse<T> {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_envelope(self.api_version, &self.request_id)
    }
}

fn validate_envelope(api_version: u16, request_id: &str) -> Result<(), ServiceError> {
    if api_version != SERVICE_API_VERSION {
        return Err(ServiceError::new(
            ServiceErrorCode::UnsupportedApiVersion,
            "The requested service API version is not supported.",
            false,
        ));
    }
    if request_id.trim().is_empty()
        || request_id.chars().count() > 128
        || request_id.chars().any(char::is_control)
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidRequest,
            "Request ID must be between 1 and 128 safe characters.",
            false,
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceErrorCode {
    InvalidRequest,
    UnsupportedApiVersion,
    FeatureDisabled,
    Unavailable,
    InvalidData,
    ProviderRateLimited,
    ProviderUnavailable,
    Timeout,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceError {
    pub code: ServiceErrorCode,
    pub safe_message: String,
    pub retryable: bool,
}

impl ServiceError {
    pub fn new(code: ServiceErrorCode, safe_message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            safe_message: safe_message.into(),
            retryable,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.safe_message.trim().is_empty()
            || self.safe_message.chars().count() > 500
            || self.safe_message.chars().any(char::is_control)
            || contains_private_location(&self.safe_message)
        {
            return Err(
                "Service error must be a safe message of at most 500 characters without local paths"
                    .into(),
            );
        }
        Ok(())
    }
}

fn contains_private_location(message: &str) -> bool {
    let bytes = message.as_bytes();
    let windows_path = bytes.windows(3).any(|segment| {
        segment[0].is_ascii_alphabetic() && segment[1] == b':' && matches!(segment[2], b'\\' | b'/')
    });
    let lower = message.to_ascii_lowercase();
    windows_path
        || message.contains('\\')
        || ["/home/", "/users/", "/private/", "/tmp/", "/var/"]
            .iter()
            .any(|prefix| lower.contains(prefix))
        || lower.contains("stack backtrace")
        || lower.contains(".rs:")
        || lower.contains(".ts:")
}

#[cfg(test)]
mod tests {
    use super::{ApiRequest, ApiResponse, SERVICE_API_VERSION, ServiceError, ServiceErrorCode};

    #[test]
    fn request_and_response_envelopes_round_trip() {
        let request = ApiRequest {
            api_version: SERVICE_API_VERSION,
            request_id: "request-1".into(),
            payload: vec![1, 2, 3],
        };
        request.validate().unwrap();
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: ApiRequest<Vec<u8>> = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, request);

        let response = ApiResponse {
            api_version: SERVICE_API_VERSION,
            request_id: request.request_id,
            result: "ok",
        };
        assert_eq!(
            serde_json::from_str::<ApiResponse<&str>>(&serde_json::to_string(&response).unwrap())
                .unwrap(),
            response
        );
        response.validate().unwrap();
    }

    #[test]
    fn version_and_safe_error_validation_fail_closed() {
        let invalid = ApiRequest {
            api_version: 2,
            request_id: "request-1".into(),
            payload: (),
        };
        assert_eq!(
            invalid.validate().unwrap_err().code,
            ServiceErrorCode::UnsupportedApiVersion
        );
        let private_path_error = ServiceError::new(
            ServiceErrorCode::Internal,
            "private path: C:\\Users\\customer\\secret.csv",
            false,
        );
        assert!(private_path_error.validate().is_err());
        assert!(
            ServiceError::new(ServiceErrorCode::Internal, "bad\nmessage", false)
                .validate()
                .is_err()
        );
        assert!(
            ServiceError::new(
                ServiceErrorCode::Internal,
                "A safe, user-facing error.",
                false
            )
            .validate()
            .is_ok()
        );
    }
}
