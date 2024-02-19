use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::USER_AGENT, request::Parts, StatusCode},
};
use uaparser::{Parser, UserAgentParser};

pub fn build_parser() -> uaparser::UserAgentParser {
    UserAgentParser::builder()
        .with_unicode_support(false)
        .build_from_bytes(include_bytes!("regexes.yaml"))
        .expect("Parser creation failed")
}

pub struct ExtractUserAgent(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractUserAgent
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user_agent) = parts.headers.get(USER_AGENT) {
            Ok(ExtractUserAgent(
                user_agent
                    .to_str()
                    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid `User-Agent` header"))?
                    .to_string(),
            ))
        } else {
            Err((StatusCode::BAD_REQUEST, "`User-Agent` header is missing"))
        }
    }
}

pub fn get_user_agent_string_short(user_agent: &str, parser: &uaparser::UserAgentParser) -> String {
    let ua = parser.parse(user_agent);
    let device = [
        ua.device.brand.unwrap_or(Default::default()),
        ua.device.family,
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect::<Vec<String>>()
    .join(" ");
    [ua.user_agent.family, ua.os.family, device.into()].join(" - ")
}
