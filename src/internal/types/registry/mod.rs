use internal::*;
use debug::*;

#[derive(Debug, Clone)]
pub struct VariantRegistry {
    pub operators: Vec<SharedString>,
    pub keywords: Vec<SharedString>,
    pub rules: Rules,
    pub has_characters: bool,
    pub has_comments: bool,
    pub has_integers: bool,
    pub has_floats: bool,
    pub has_strings: bool,
    pub has_negatives: bool,
}

impl VariantRegistry {

    pub fn new() -> Self {
        Self {
            operators: Vec::new(),
            keywords: Vec::new(),
            rules: Rules::new(),
            has_characters: false,
            has_comments: false,
            has_integers: false,
            has_floats: false,
            has_strings: false,
            has_negatives: false,
        }
    }

    pub fn serialize(self) -> Data {
        let mut map = Map::new();

        map.insert(identifier!("characters"), boolean!(self.has_characters));
        map.insert(identifier!("comments"), boolean!(self.has_comments));
        map.insert(identifier!("integers"), boolean!(self.has_integers));
        map.insert(identifier!("floats"), boolean!(self.has_floats));
        map.insert(identifier!("strings"), boolean!(self.has_strings));
        map.insert(identifier!("negatives"), boolean!(self.has_negatives));

        let operator_list = self.operators.into_iter().map(|operator| identifier!(String, operator)).collect();
        map.insert(identifier!("operators"), list!(operator_list));

        let keyword_list = self.keywords.into_iter().map(|keyword| identifier!(String, keyword)).collect();
        map.insert(identifier!("keywords"), list!(keyword_list));

        let rules_map = self.rules.serialize();
        map.insert(identifier!("rules"), rules_map);

        return map!(map);
    }

    pub fn deserialize(serialized: &Data) -> Status<Self> {

        let mut variant_registry = VariantRegistry::new();

        let has_characters = confirm!(serialized.index(&identifier!("characters")));
        let has_characters = expect!(has_characters, string!("variant registry may not be missing characters field"));
        variant_registry.has_characters = unpack_boolean!(&has_characters);

        let has_comments = confirm!(serialized.index(&identifier!("comments")));
        let has_comments = expect!(has_comments, string!("variant registry may not be missing comments field"));
        variant_registry.has_comments = unpack_boolean!(&has_comments);

        let has_integers = confirm!(serialized.index(&identifier!("integers")));
        let has_integers = expect!(has_integers, string!("variant registry may not be missing integers field"));
        variant_registry.has_integers = unpack_boolean!(&has_integers);

        let has_floats = confirm!(serialized.index(&identifier!("floats")));
        let has_floats = expect!(has_floats, string!("variant registry may not be missing floats field"));
        variant_registry.has_floats = unpack_boolean!(&has_floats);

        let has_strings = confirm!(serialized.index(&identifier!("strings")));
        let has_strings = expect!(has_strings, string!("variant registry may not be missing strings field"));
        variant_registry.has_strings = unpack_boolean!(&has_strings);

        let has_negatives = confirm!(serialized.index(&identifier!("negatives")));
        let has_negatives = expect!(has_negatives, string!("variant registry may not be missing negatives field"));
        variant_registry.has_negatives = unpack_boolean!(&has_negatives);

        let operator_list = confirm!(serialized.index(&identifier!("operators")));
        let operator_list = expect!(operator_list, string!("variant registry may not be missing operators field"));

        for operator in unpack_list!(&operator_list).iter() {
            variant_registry.operators.push(unpack_identifier!(operator));
        }

        let keyword_list = confirm!(serialized.index(&identifier!("keywords")));
        let keyword_list = expect!(keyword_list, string!("variant registry may not be missing keywords field"));

        for keyword in unpack_list!(&keyword_list).iter() {
            variant_registry.keywords.push(unpack_identifier!(keyword));
        }

        let rules = confirm!(serialized.index(&identifier!("rules")));
        let rules = expect!(rules, string!("variant registry may not be missing rules field"));
        variant_registry.rules = confirm!(Rules::deserialize(&rules));

        return success!(variant_registry);
    }

    //pub fn check_prefix(&self, string: &str) -> Option<(String, Action)> {
    //    return self.rules.check_prefix(string); // ALSO CHECK IF STRING IS PURE
    //}

    pub fn set_rules(&mut self, rules: Rules) {
        self.rules = rules;
    }

    pub fn validate_operators(&self, filters: &Vec<SharedString>) -> Status<()> {
        ensure!(!self.operators.is_empty(), string!("tokenizer does not support operators"));
        for filter in filters.iter() {
            ensure!(self.is_operator(filter), string!("{} is not a valid operator", filter));
        }
        return success!(());
    }

    pub fn validate_keywords(&self, filters: &Vec<SharedString>) -> Status<()> {
        ensure!(!self.keywords.is_empty(), string!("tokenizer does not support keywords"));
        for filter in filters.iter() {
            ensure!(self.is_keyword(filter), string!("{} is not a valid keyword", filter));
        }
        return success!(());
    }

    pub fn validate_identifiers(&self, filters: &Vec<SharedString>) -> Status<()> {
        ensure!(self.has_identifiers(), string!("tokenizer does not support identifiers"));
        for filter in filters.iter() {
            ensure!(self.is_identifier(filter), string!("{} is not a valid identifier", filter));
        }
        return success!(());
    }

    pub fn validate_type_identifiers(&self, filters: &Vec<SharedString>) -> Status<()> {
        ensure!(self.has_type_identifiers(), string!("tokenizer does not support type identifiers"));
        for filter in filters.iter() {
            ensure!(self.is_type_identifier(filter), string!("{} is not a valid type identifier", filter));
        }
        return success!(());
    }

    pub fn validate_integers(&self, filters: &Vec<i64>) -> Status<()> {
        ensure!(self.has_integers, string!("tokenizer does not support integers"));
        for filter in filters.iter() {
            ensure!(*filter > 0 || self.has_negatives, string!("tokenizer does not support negative integers"));
        }
        return success!(());
    }

    pub fn validate_floats(&self, filters: &Vec<f64>) -> Status<()> {
        ensure!(self.has_floats, string!("tokenizer does not support floats"));
        for filter in filters.iter() {
            ensure!(*filter > 0.0 || self.has_negatives, string!("tokenizer does not support negative floats"));
        }
        return success!(());
    }

    pub fn validate_strings(&self) -> Status<()> {
        ensure!(self.has_strings, string!("tokenizer does not support strings"));
        return success!(());
    }

    pub fn validate_characters(&self) -> Status<()> {
        ensure!(self.has_characters, string!("tokenizer does not support characters"));
        return success!(());
    }

    pub fn has_identifiers(&self) -> bool {
        return self.rules.has_mapping("identifier");
    }

    pub fn has_type_identifiers(&self) -> bool {
        return self.rules.has_mapping("type_identifier");
    }

    pub fn is_identifier(&self, source: &SharedString) -> bool {
        match self.rules.check_prefix(source) {
            Some((_, action)) => return action.is_mapped_to("identifier"),
            None => return false,
        }
    }

    pub fn is_type_identifier(&self, source: &SharedString) -> bool {
        match self.rules.check_prefix(source) {
            Some((_, action)) => return action.is_mapped_to("type_identifier"),
            None => return false,
        }
    }

    pub fn is_operator(&self, compare: &SharedString) -> bool {
        return self.operators.iter().find(|operator| **operator == *compare).is_some();
    }

    pub fn is_keyword(&self, compare: &SharedString) -> bool {
        return self.keywords.iter().find(|keyword| **keyword == *compare).is_some();
    }

    pub fn register_operator(&mut self, operator: SharedString) {
        if !self.is_operator(&operator) {
            self.operators.push(operator);
        }
    }

    pub fn register_keyword(&mut self, keyword: SharedString) {
        if !self.is_keyword(&keyword) {
            self.keywords.push(keyword);
        }
    }

    pub fn avalible_keywords(&self) -> Vec<SharedString> {
        return self.keywords.clone();
    }

    pub fn avalible_operators(&self) -> Vec<SharedString> {
        return self.operators.clone();
    }
}
