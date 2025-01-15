use super::kinds::ServiceItemKind;

pub trait Content {
    fn title(&self) -> String;
    fn kind(&self) -> ServiceItemKind;
}
