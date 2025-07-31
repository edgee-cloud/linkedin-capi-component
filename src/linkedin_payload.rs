use anyhow::anyhow;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::exports::edgee::components::data_collection::{Consent, Dict, Event};

#[derive(Serialize, Debug, Default)]
pub(crate) struct LinkedinPayload {
    pub data: LinkedinEvent,
    #[serde(skip)]
    pub access_token: String,
}

impl LinkedinPayload {
    pub fn new(settings: Dict) -> anyhow::Result<Self> {
        let cred: HashMap<String, String> = settings
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let access_token = match cred.get("linkedin_access_token") {
            Some(key) => key,
            None => return Err(anyhow!("Missing LinkedIn Access Token")),
        }
        .to_string();

        Ok(Self {
            access_token,
            ..LinkedinPayload::default()
        })
    }
}

/// LinkedIn Event
///
/// This is the event that will be sent to LinkedIn CAPI.
/// To know more about the event structure, check the online documentation: https://learn.microsoft.com/en-us/linkedin/marketing/integrations/ads-reporting/conversions-api
///
/// There is one way of tracking conversions using this component:
/// - Personalized events, which are user actions defined by you as conversation rules on the linkedin api interface and recorded by calling by calling a `track`event with a custom event name.
#[derive(Serialize, Debug, Default)]
pub struct LinkedinEvent {
    pub conversion: String,
    #[serde(rename = "conversionHappenedAt")]
    pub event_time: i64,
    #[serde(rename = "user")]
    pub user_data: UserData,
    #[serde(rename = "eventId")]
    pub event_id: String,
}

// User Data
//
// This is the user data that will be sent to LinkedIn CAPI.
// To know more about the user data structure, check the online documentation: https://learn.microsoft.com/en-us/linkedin/marketing/integrations/ads-reporting/conversions-api?view=li-lms-2024-11&tabs=http#conversioneventuser
#[derive(Serialize, Debug, Default)]
pub struct UserData {
    #[serde(rename = "userIds")]
    pub user_ids: Vec<UserId>,
    #[serde(rename = "externalIds")]
    pub external_ids: Vec<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct UserId {
    #[serde(rename = "idType")]
    pub id_type: String,
    #[serde(rename = "idValue")]
    pub id_value: String,
}

impl LinkedinEvent {
    pub fn new(
        edgee_event: &Event,
        event_name: &str,
        event_id: &str,
        li_fat_id: Option<&str>,
    ) -> anyhow::Result<Self> {
        // Default LinkedIn event

        let mut linkedin_event = LinkedinEvent {
            conversion: event_name.to_string(),
            event_time: edgee_event.timestamp_millis,
            event_id: event_id.to_string(),
            user_data: UserData::default(),
        };

        let mut user_data = UserData {
            ..UserData::default()
        };

        let user_properties = edgee_event.context.user.properties.clone();
        user_data
            .external_ids
            .push(edgee_event.context.user.user_id.clone());

        user_data.user_ids.push(UserId {
            id_type: "LINKEDIN_FIRST_PARTY_ADS_TRACKING_UUID".to_owned(),
            id_value: li_fat_id.unwrap_or("").to_string(),
        });

        for (key, value) in user_properties.iter() {
            match key.as_str() {
                "email" => user_data.user_ids.push(UserId {
                    id_type: "SHA256_EMAIL".to_owned(),
                    id_value: hash_value(value),
                }),
                _ => {
                    // do nothing
                }
            }
        }

        if edgee_event.consent.is_some() && edgee_event.consent.unwrap() != Consent::Granted {
            // Consent is not granted, so we don't send the event
            return Err(anyhow!("Consent is not granted"));
        }

        if user_data.user_ids.is_empty() {
            return Err(anyhow!("User properties must contain email"));
        }

        linkedin_event.user_data = user_data;

        Ok(linkedin_event)
    }
}

/// SHA256 hash value
///
/// This function is used to hash the value.
pub(crate) fn hash_value(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_value() {
        let input = "test@example.com";
        let expected = "973dfe463ec85785f5f95af5ba3906eedb2d931c24e69824a89ea65dba4e813b";
        assert_eq!(hash_value(input), expected);
    }

    #[test]
    fn test_hash_value_empty_string() {
        let input = "";
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash_value(input), expected);
    }

    #[test]
    fn test_linkedin_payload_new_success() {
        let settings = vec![(
            "linkedin_access_token".to_string(),
            "test_token".to_string(),
        )];

        let payload = LinkedinPayload::new(settings).unwrap();
        assert_eq!(payload.access_token, "test_token");
    }

    #[test]
    fn test_linkedin_payload_new_missing_token() {
        let settings = vec![];

        let result = LinkedinPayload::new(settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing LinkedIn Access Token"));
    }

    #[test]
    fn test_user_id_creation() {
        let user_id = UserId {
            id_type: "SHA256_EMAIL".to_owned(),
            id_value: "hashed_email".to_owned(),
        };

        assert_eq!(user_id.id_type, "SHA256_EMAIL");
        assert_eq!(user_id.id_value, "hashed_email");
    }
}
