use plegine::json::{self, parse_type_err_res, FromValue};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TransportGroup {
    Gas,
    Liquid,
    Solid,
}

impl FromValue for TransportGroup {
    fn from_value(value: json::Value) -> plegine::json::ParseResult<Self> {
        match value {
            json::Value::String(string) if string == "gas" => Ok(TransportGroup::Gas),
            json::Value::String(string) if string == "liquid" => Ok(TransportGroup::Liquid),
            json::Value::String(string) if string == "solid" => Ok(TransportGroup::Solid),
            _ => parse_type_err_res(),
        }
    }
}
