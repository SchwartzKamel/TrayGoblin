use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, de::IgnoredAny};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    TurnStart,
    TurnEnd,
    ModelChange,
    AssistantMessage,
    ToolExecutionComplete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionEvent {
    pub kind: EventKind,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub model: Option<String>,
    pub success: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MalformedEvent {
    InvalidJson,
    InvalidTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParsedEvent {
    Known(SessionEvent),
    Unsupported,
    Malformed(MalformedEvent),
}

#[derive(Debug, Default, Deserialize)]
struct EventData {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    success: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TimestampField {
    String(String),
    Unsupported(IgnoredAny),
}

#[derive(Deserialize)]
struct EventEnvelope {
    #[serde(rename = "type", alias = "eventType")]
    event_type: String,
    #[serde(default)]
    timestamp: Option<TimestampField>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    data: EventData,
}

pub fn parse_event_line(line: &str) -> ParsedEvent {
    let envelope = match serde_json::from_str::<EventEnvelope>(line) {
        Ok(envelope) => envelope,
        Err(_) => return ParsedEvent::Malformed(MalformedEvent::InvalidJson),
    };

    let kind = match envelope.event_type.as_str() {
        "assistant.turn_start" => EventKind::TurnStart,
        "assistant.turn_end" => EventKind::TurnEnd,
        "session.model_change" => EventKind::ModelChange,
        "assistant.message" => EventKind::AssistantMessage,
        "tool.execution_complete" => EventKind::ToolExecutionComplete,
        _ => return ParsedEvent::Unsupported,
    };

    let timestamp = match envelope.timestamp {
        Some(TimestampField::String(timestamp)) => match DateTime::parse_from_rfc3339(&timestamp) {
            Ok(timestamp) => Some(timestamp),
            Err(_) => return ParsedEvent::Malformed(MalformedEvent::InvalidTimestamp),
        },
        Some(TimestampField::Unsupported(_)) => {
            return ParsedEvent::Malformed(MalformedEvent::InvalidTimestamp);
        }
        None => None,
    };

    ParsedEvent::Known(SessionEvent {
        kind,
        timestamp,
        model: normalize_model(envelope.model.or(envelope.data.model)),
        success: envelope.success.or(envelope.data.success),
    })
}

pub fn latest_model<'a>(events: impl IntoIterator<Item = &'a SessionEvent>) -> Option<&'a str> {
    events
        .into_iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ModelChange | EventKind::AssistantMessage
            )
        })
        .filter_map(|event| event.model.as_deref())
        .last()
}

fn normalize_model(model: Option<String>) -> Option<String> {
    model.and_then(|model| {
        let model = model.trim();
        (!model.is_empty()).then(|| model.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        EventKind, MalformedEvent, ParsedEvent, SessionEvent, latest_model, parse_event_line,
    };

    const EVENTS: &str = include_str!("../tests/fixtures/parser/events.jsonl");
    const MALFORMED_EVENTS: &str = include_str!("../tests/fixtures/parser/malformed-events.jsonl");

    fn known_events() -> Vec<SessionEvent> {
        EVENTS
            .lines()
            .filter_map(|line| match parse_event_line(line) {
                ParsedEvent::Known(event) => Some(event),
                ParsedEvent::Unsupported | ParsedEvent::Malformed(_) => None,
            })
            .collect()
    }

    // This protects the model tooltip from becoming stale across either supported metadata event.
    #[test]
    fn tracks_latest_model() {
        let events = known_events();

        assert_eq!(latest_model(&events), Some("gpt-5.6-sol"));
    }

    // This keeps additions to Copilot's internal event vocabulary from stopping monitoring.
    #[test]
    fn unknown_future_event_is_non_fatal() {
        let outcome = EVENTS
            .lines()
            .map(parse_event_line)
            .find(|outcome| matches!(outcome, ParsedEvent::Unsupported));

        assert_eq!(outcome, Some(ParsedEvent::Unsupported));
    }

    // This is the deterministic privacy contract: ignored JSON fields never enter the typed model.
    #[test]
    fn does_not_model_sensitive_fields() {
        let parsed: Vec<_> = EVENTS.lines().map(parse_event_line).collect();
        let modeled = format!("{parsed:?}");

        for forbidden in [
            "SENSITIVE_SENTINEL",
            "prompt",
            "assistantContent",
            "toolArguments",
            "toolResult",
            "tokens",
            "credential",
            "repositoryFileContents",
        ] {
            assert!(
                !modeled.contains(forbidden),
                "modeled forbidden field: {forbidden}"
            );
        }
    }

    // This proves corrupt lines and invalid timestamps are classified without exposing raw input.
    #[test]
    fn malformed_lines_are_non_fatal() {
        let outcomes: Vec<_> = MALFORMED_EVENTS.lines().map(parse_event_line).collect();

        assert_eq!(
            outcomes,
            [
                ParsedEvent::Malformed(MalformedEvent::InvalidJson),
                ParsedEvent::Malformed(MalformedEvent::InvalidTimestamp),
            ]
        );
    }

    // This protects the exact turn and tool fields consumed by the polling monitor.
    #[test]
    fn extracts_state_timestamps_and_success() {
        let events = known_events();

        assert_eq!(events[0].kind, EventKind::TurnStart);
        assert_eq!(
            events[0].timestamp.unwrap().to_rfc3339(),
            "2026-07-28T07:20:00+00:00"
        );
        let failed_tool = events
            .iter()
            .find(|event| event.kind == EventKind::ToolExecutionComplete)
            .expect("fixture should contain a tool completion");
        assert_eq!(failed_tool.success, Some(false));
        assert!(events.iter().any(|event| event.kind == EventKind::TurnEnd));
    }
}
