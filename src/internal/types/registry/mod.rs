use internal::*;

#[derive(Debug)]
pub struct VariantRegistry {
    operators:                  Vec<VectorString>,
    keywords:                   Vec<VectorString>,
    rules:                      Rules,
    pub has_characters:         bool,
    pub has_comments:           bool,
    pub has_integers:           bool,
    pub has_floats:             bool,
    pub has_strings:            bool,
    pub has_negatives:          bool,
}

impl VariantRegistry {

    pub fn new() -> Self {
        Self {
            operators:              Vec::new(),
            keywords:               Vec::new(),
            rules:                  Rules::new(),
            has_characters:         false,
            has_comments:           false,
            has_integers:           false,
            has_floats:             false,
            has_strings:            false,
            has_negatives:          false,
        }
    }

    //pub fn check_prefix(&self, string: &str) -> Option<(String, Action)> {
    //    return self.rules.check_prefix(string); // ALSO CHECK IF STRING IS PURE
    //}

    pub fn set_rules(&mut self, rules: Rules) {
        self.rules = rules;
    }

    pub fn validate_operators(&self, filters: &Vec<VectorString>) -> Status<()> {
        ensure!(!self.operators.is_empty(), Message, string!(str, "tokenizer does not support operators"));
        for filter in filters.iter() {
            ensure!(self.is_operator(filter), Message, string!(str, "{} is not a valid operator", filter));
        }
        return success!(());
    }

    pub fn validate_keywords(&self, filters: &Vec<VectorString>) -> Status<()> {
        ensure!(!self.keywords.is_empty(), Message, string!(str, "tokenizer does not support keywords"));
        for filter in filters.iter() {
            ensure!(self.is_keyword(filter), Message, string!(str, "{} is not a valid keyword", filter));
        }
        return success!(());
    }

    pub fn validate_identifiers(&self, filters: &Vec<VectorString>) -> Status<()> {
        ensure!(self.has_identifiers(), Message, string!(str, "tokenizer does not support identifiers"));
        for filter in filters.iter() {
            ensure!(self.is_identifier(filter), Message, string!(str, "{} is not a valid identifier", filter));
        }
        return success!(());
    }

    pub fn validate_type_identifiers(&self, filters: &Vec<VectorString>) -> Status<()> {
        ensure!(self.has_type_identifiers(), Message, string!(str, "tokenizer does not support type identifiers"));
        for filter in filters.iter() {
            ensure!(self.is_type_identifier(filter), Message, string!(str, "{} is not a valid type identifier", filter));
        }
        return success!(());
    }

    pub fn validate_integers(&self, filters: &Vec<i64>) -> Status<()> {
        ensure!(self.has_integers, Message, string!(str, "tokenizer does not support integers"));
        for filter in filters.iter() {
            ensure!(*filter > 0 || self.has_negatives, Message, string!(str, "tokenizer does not support negative integers"));
        }
        return success!(());
    }

    pub fn validate_floats(&self, filters: &Vec<f64>) -> Status<()> {
        ensure!(self.has_floats, Message, string!(str, "tokenizer does not support floats"));
        for filter in filters.iter() {
            ensure!(*filter > 0.0 || self.has_negatives, Message, string!(str, "tokenizer does not support negative floats"));
        }
        return success!(());
    }

    pub fn validate_strings(&self) -> Status<()> {
        ensure!(self.has_strings, Message, string!(str, "tokenizer does not support strings"));
        return success!(());
    }

    pub fn validate_characters(&self) -> Status<()> {
        ensure!(self.has_characters, Message, string!(str, "tokenizer does not support characters"));
        return success!(());
    }

    pub fn has_identifiers(&self) -> bool {
        return self.rules.has_mapping("identifier");
    }

    pub fn has_type_identifiers(&self) -> bool {
        return self.rules.has_mapping("type_identifier");
    }

    pub fn is_identifier(&self, source: &VectorString) -> bool {
        match self.rules.check_prefix(source) {
            Some((_, action)) => return action.is_mapped_to("identifier"),
            None => return false,
        }
    }

    pub fn is_type_identifier(&self, source: &VectorString) -> bool {
        match self.rules.check_prefix(source) {
            Some((_, action)) => return action.is_mapped_to("type_identifier"),
            None => return false,
        }
    }

    pub fn is_operator(&self, compare: &VectorString) -> bool {
        return self.operators.iter().find(|operator| **operator == *compare).is_some();
    }

    pub fn is_keyword(&self, compare: &VectorString) -> bool {
        return self.keywords.iter().find(|keyword| **keyword == *compare).is_some();
    }

    pub fn register_operator(&mut self, operator: VectorString) {
        if !self.is_operator(&operator) {
            self.operators.push(operator);
        }
    }

    pub fn register_keyword(&mut self, keyword: VectorString) {
        if !self.is_keyword(&keyword) {
            self.keywords.push(keyword);
        }
    }

    pub fn avalible_keywords(&self) -> Vec<VectorString> {
        return self.keywords.clone();
    }

    pub fn avalible_operators(&self) -> Vec<VectorString> {
        return self.operators.clone();
    }
}
