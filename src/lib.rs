mod linkedin_payload;

use crate::exports::edgee::protocols::data_collection::{
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
