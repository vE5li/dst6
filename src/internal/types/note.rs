use internal::*;
use debug::*;

#[derive(Clone, Debug)]
pub struct Note {
    pub kind: Data,
    pub message: SharedString,
    pub position: Position,
}

impl Note {

    pub fn new(kind: Data, message: SharedString, position: Position) -> Self {
        Self {
            kind: kind,
            message: message,
            position: position,
        }
    }

    pub fn serialize(self) -> Data {
        let mut map = Map::new();
        map.insert(identifier!("type"), self.kind);
        map.insert(identifier!("message"), string!(String, self.message));
        map.insert(identifier!("position"), self.position.serialize());
        return map!(map);
    }
}
