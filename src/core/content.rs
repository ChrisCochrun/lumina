use super::{kinds::ServiceItemKind, service_items::ServiceItem};

pub trait Content {
    fn title(&self) -> String;
    fn kind(&self) -> ServiceItemKind;
    fn to_service_item(&self) -> ServiceItem;
}
