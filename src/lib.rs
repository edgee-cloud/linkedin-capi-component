mod linkedin_payload;

use crate::exports::edgee::components::data_collection::{
    Data, Dict, EdgeeRequest, Event, Guest, HttpMethod,
};
use linkedin_payload::{LinkedinEvent, LinkedinPayload};

wit_bindgen::generate!({world: "data-collection", path: "wit", generate_all});

export!(LinkedinComponent);

struct LinkedinComponent;

impl Guest for LinkedinComponent {
    fn page(_edgee_event: Event, _settings: Dict) -> Result<EdgeeRequest, String> {
        Err("Page event not implemented for this component".to_string())
    }

    fn track(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Track name should be set to your conversion rule. ex: urn:lla:llaPartnerConversion:123".to_string());
            }

            let mut linkedin_payload = LinkedinPayload::new(settings).map_err(|e| e.to_string())?;
            let event =
                LinkedinEvent::new(&edgee_event, data.name.as_str()).map_err(|e| e.to_string())?;

            linkedin_payload.data = event;

            Ok(build_edgee_request(linkedin_payload))
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(_edgee_event: Event, _settings: Dict) -> Result<EdgeeRequest, String> {
        Err("User event not implemented for this component".to_string())
    }
}

fn build_edgee_request(linkedin_payload: LinkedinPayload) -> EdgeeRequest {
    let headers = vec![
        (
            String::from("content-type"),
            String::from("application/json"),
        ),
        (
            String::from("X-Restli-Protocol-Version"),
            String::from("2.0.0"),
        ),
        (String::from("LinkedIn-Version"), String::from("202411")),
        (
            String::from("Authorization"),
            format!("Bearer {}", linkedin_payload.access_token),
        ),
    ];

    let url = "https://api.linkedin.com/rest/conversionEvents";

    EdgeeRequest {
        method: HttpMethod::Post,
        url: url.to_string(),
        headers,
        forward_client_headers: true,
        body: serde_json::to_string(&linkedin_payload.data).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::components::data_collection::{
        Campaign, Client, Context, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::components::data_collection::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id,
            properties: vec![
                ("email".to_string(), "test@test.com".to_string()),
                ("phone_number".to_string(), "+39 1231231231".to_string()),
                ("first_name".to_string(), "John".to_string()),
                ("last_name".to_string(), "Doe".to_string()),
                ("gender".to_string(), "Male".to_string()),
                ("date_of_birth".to_string(), "1979-12-31".to_string()),
                ("city".to_string(), "Las Vegas".to_string()),
                ("state".to_string(), "Nevada".to_string()),
                ("zip_code".to_string(), "11111".to_string()),
                ("country".to_string(), "USA".to_string()),
                ("random_property".to_string(), "abc".to_string()), // will be ignored
            ],
        }
    }

    fn sample_user_data_invalid_without_ids() -> UserData {
        UserData {
            user_id: "".to_string(),
            anonymous_id: "".to_string(),
            edgee_id: "abc".to_string(),
            properties: vec![],
        }
    }

    fn sample_user_data_invalid_without_email() -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id: "abc".to_string(),
            properties: vec![
                // both missing
                //("email".to_string(), "test@test.com".to_string()),
                ("phone_number".to_string(), "+39 1231231231".to_string()),
                ("first_name".to_string(), "John".to_string()),
                ("last_name".to_string(), "Doe".to_string()),
                ("gender".to_string(), "Male".to_string()),
                ("date_of_birth".to_string(), "1979-12-31".to_string()),
                ("city".to_string(), "Las Vegas".to_string()),
                ("state".to_string(), "Nevada".to_string()),
                ("zip_code".to_string(), "11111".to_string()),
                ("country".to_string(), "USA".to_string()),
                ("random_property".to_string(), "abc".to_string()), // will be ignored
            ],
        }
    }

    fn sample_context(edgee_id: String, locale: String, session_start: bool) -> Context {
        Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale,
                timezone: "CET".to_string(),
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "fuck knows".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "abc".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "mobile".to_string(),
                user_agent_model: "don't know".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start,
                first_seen: 123,
                last_seen: 123,
            },
        }
    }

    fn sample_page_data() -> PageData {
        PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("prop3".to_string(), "true".to_string()),
                ("prop4".to_string(), "false".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_track_data(event_name: String) -> TrackData {
        TrackData {
            name: event_name,
            products: vec![],
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_user_event_without_ids(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data_invalid_without_ids();
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_user_event_without_email(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data_invalid_without_email();
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_settings() -> Vec<(String, String)> {
        vec![
            ("linkedin_access_token".to_string(), "abc".to_string()),
            ("pinterest_ad_account_id".to_string(), "abc".to_string()),
        ]
    }

    #[test]
    fn page_with_consent() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_empty_consent() {
        let event = sample_page_event(
            None, // no consent at all -> works fine
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_consent_denied_fails() {
        let event = sample_page_event(
            Some(Consent::Denied),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_with_edgee_id_uuid() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "fr".to_string(), true);
        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_with_empty_locale() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), true);

        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_not_session_start() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), false);
        let settings = sample_settings();
        let result = LinkedinComponent::page(event, settings);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Page event not implemented for this component"),
            true
        );
    }

    #[test]
    fn page_without_access_token_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let settings: Vec<(String, String)> = vec![]; // empty
        let result = LinkedinComponent::page(event, settings); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn page_without_pixel_id_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let settings: Vec<(String, String)> = vec![
            ("linkedin_access_token".to_string(), "abc".to_string()), // only access token
        ];
        let result = LinkedinComponent::page(event, settings); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn track_with_consent() {
        let event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::track(event, settings);
        println!("{:?}", result);
        assert_eq!(result.clone().is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn track_with_empty_name_fails() {
        let event = sample_track_event(
            "".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::track(event, settings);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn user_event() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::user(event, settings);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("User event not implemented for this component"),
            true
        );
    }

    #[test]
    fn user_event_without_ids_fails() {
        let event = sample_user_event_without_ids(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::user(event, settings);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("User event not implemented for this component"),
            true
        );
    }

    #[test]
    fn user_event_without_email_or_phone_fails() {
        let event = sample_user_event_without_email(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = LinkedinComponent::user(event, settings);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("User event not implemented for this component"),
            true
        );
    }

    #[test]
    fn track_event_without_user_context_properties_fails() {
        let mut event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        event.context.user.properties = vec![]; // empty context user properties
        event.context.user.user_id = "".to_string(); // empty context user id
        let settings = sample_settings();
        let result = LinkedinComponent::track(event, settings);
        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("User properties are empty"),
            true
        );
    }
}
