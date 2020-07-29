mod action;

use internal::*;
use debug::*;

pub use self::action::Action;

#[derive(Debug, Clone)]
pub struct Rules {
    rules:      Vec<(SharedString, Action)>,
}

impl Rules {

    pub fn new() -> Self {
        Self {
            rules:      Vec::new(),
        }
    }

    pub fn serialize(self) -> Data {
        let mut map = Map::new();

        for (signature, action) in self.rules.into_iter() {
            map.insert(string!(String, signature), action.serialize());
        }

        return map!(map);
    }

    pub fn deserialize(serialized: &Data) -> Status<Self> {
        let mut rules = Self::new();

        for (signature, action) in confirm!(serialized.pairs()).iter() {
            let signature = unpack_string!(signature);
            let action = confirm!(Action::deserialize(action));
            rules.add(signature, action);
        }

        return success!(rules);
    }

    pub fn has_mapping(&self, string: &str) -> bool {
        return self.rules.iter().find(|(_, action)| action.is_mapped_to(string)).is_some();
    }

    pub fn has_mapping_to(&self, source_signature: &SharedString, string: &str) -> bool {
        return self.rules.iter().find(|(signature, action)| *source_signature == *signature && action.is_mapped_to(string)).is_some();
    }

    fn contains(&self, new: &SharedString) -> bool {
        for (pattern, _rule) in self.rules.iter() {
            if pattern == new {
                return true;
            }
        }
        return false;
    }

    pub fn add(&mut self, pattern: SharedString, action: Action) -> Status<()> {
        if self.contains(&pattern) {
            return error!(DuplicateSignature, Data::String(pattern));
        }
        match self.rules.iter().position(|(current_pattern, _)| current_pattern.len() <= pattern.len()) {
            Some(index) => self.rules.insert(index, (pattern, action)),
            None => self.rules.push((pattern, action)),
        }
        return success!(());
    }

    pub fn check_stack(&self, stack: &mut CharacterStack) -> Option<(SharedString, Action)> {
        for (pattern, action) in self.rules.iter() {
            if stack.check_string(pattern) {
                 return Some((pattern.clone(), action.clone()));
            }
        }
        return None;
    }

    pub fn check_word(&self, string: &SharedString) -> Option<(SharedString, Action)> {
        for (pattern, action) in self.rules.iter() {
            if *pattern == *string {
                return Some((pattern.clone(), action.clone()));
            }
        }
        return None;
    }

    pub fn check_prefix(&self, string: &SharedString) -> Option<(SharedString, Action)> {
        for (pattern, action) in self.rules.iter() {
            if let Some(position) = string.find(pattern) {
                if position == 0 {
                    return Some((pattern.clone(), action.clone()));
                }
            }
        }
        return None;
    }
}
