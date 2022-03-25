use std::sync::Arc;

use serde_json::{Value, Map};

pub enum EventType {
    ObjectCreated,
    ObjectRemoved,
}

pub struct Message {
    pub event_type: EventType,
    pub key: String,
}

pub fn to_messages(value: &Value) -> Option<Vec<Arc<Message>>> {
    let object = value.as_object()?;
    let records = object.get("Records")?.as_array()?;

    let mut result = Vec::new();
    for rec in records {
        if let Some(message) = to_message(rec.as_object()?) {
            result.push(message);
        }
    }
    Some(result)
}

fn to_message(value: &Map<String, Value>) -> Option<Arc<Message>> {
    let event_source = value.get("eventSource")?.as_str()?;
    if event_source != "aws:s3" {
        return None;
    }

    let event_name = value.get("eventName")?.as_str()?;

    let event_type = if event_name.starts_with("ObjectCreated") {
        EventType::ObjectCreated
    } else if event_name.starts_with("ObjectRemoved") {
        EventType::ObjectRemoved
    } else {
        return None;
    };

    let s3 = value.get("s3")?.as_object()?;
    let object = s3.get("object")?.as_object()?;
    let key = object.get("key")?.as_str()?;

    Some(Arc::new(Message {
        event_type,
        key: key.into(),
    }))
}
