use serde_json::Value;

const MASK: &str = "[REDACTED]";
const SECRET_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "refresh_token",
    "refreshToken",
    "id_token",
    "idToken",
    "OPENAI_API_KEY",
    "api_key",
    "authorization",
    "Authorization",
];

pub fn mask_header(name: &str, value: &str) -> String {
    if name.eq_ignore_ascii_case("authorization") {
        MASK.to_string()
    } else {
        value.to_string()
    }
}

pub fn mask_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, item) in map.iter_mut() {
                if SECRET_KEYS.iter().any(|secret| secret == key) {
                    *item = Value::String(MASK.to_string());
                } else {
                    mask_json(item);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                mask_json(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn masks_authorization_header() {
        assert_eq!(mask_header("Authorization", "Bearer secret"), "[REDACTED]");
        assert_eq!(
            mask_header("content-type", "application/json"),
            "application/json"
        );
    }

    #[test]
    fn masks_token_like_json_fields() {
        let mut value = json!({
            "Authorization": "Bearer secret",
            "OPENAI_API_KEY": "synthetic-api-key",
            "tokens": {
                "access_token": "access",
                "refresh_token": "refresh",
                "id_token": "id",
                "account_id": "not-secret"
            },
            "claudeAiOauth": {
                "accessToken": "access",
                "refreshToken": "refresh"
            }
        });

        mask_json(&mut value);

        assert_eq!(value["Authorization"], "[REDACTED]");
        assert_eq!(value["OPENAI_API_KEY"], "[REDACTED]");
        assert_eq!(value["tokens"]["access_token"], "[REDACTED]");
        assert_eq!(value["tokens"]["refresh_token"], "[REDACTED]");
        assert_eq!(value["tokens"]["id_token"], "[REDACTED]");
        assert_eq!(value["tokens"]["account_id"], "not-secret");
        assert_eq!(value["claudeAiOauth"]["accessToken"], "[REDACTED]");
        assert_eq!(value["claudeAiOauth"]["refreshToken"], "[REDACTED]");
    }
}
