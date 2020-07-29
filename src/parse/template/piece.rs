use internal::*;
use debug::*;

use parse::{ Decision, Templates };

macro_rules! filters {
    ($piece_stack:expr, $extractor:ident) => ({
        let mut filters = Vec::new();
        if let Some(filter_source) = $piece_stack.pop() {
            for filter in unpack_list!(&filter_source).iter() {
                filters.push($extractor!(filter));
            }
        }
        filters
    });
}

macro_rules! typed_token_list { // TODO: clean up
    (Operator, $list:expr, $filters:expr, $registry:expr) => ({
        if $filters.is_empty() {
            for operator in $registry.avalible_operators().iter() {
                let location = identifier!("operator:{}", operator);
                if !$list.contains(&location) {
                    $list.push(location);
                }
            }
        } else {
            for operator in $filters.iter() {
                let location = identifier!("operator:{}", operator);
                if !$list.contains(&location) {
                    $list.push(location);
                }
            }
        }
        true
    });
    (Keyword, $list:expr, $filters:expr, $registry:expr) => ({
        if $filters.is_empty() {
            for keyword in $registry.avalible_keywords().iter() {
                let location = identifier!("keyword:{}", keyword);
                if !$list.contains(&location) {
                    $list.push(location);
                }
            }
        } else {
            for keyword in $filters.iter() {
                let location = identifier!("keyword:{}", keyword);
                if !$list.contains(&location) {
                    $list.push(location);
                }
            }
        }
        true
    });
}

#[derive(Debug, Clone)]
pub enum Piece {
    Data(Data, Data),
    Comment(Data),
    Merge(Vec<Data>),
    Template(Option<Data>, Vec<Data>),
    List(Option<Data>, Box<Piece>, Box<Option<Piece>>),
    Confirmed(Option<Data>, Box<Piece>, Box<Option<Piece>>),
    Keyword(Option<Data>, Vec<SharedString>),
    Operator(Option<Data>, Vec<SharedString>),
    Identifier(Option<Data>, Vec<SharedString>),
    TypeIdentifier(Option<Data>, Vec<SharedString>),
    String(Option<Data>, Vec<SharedString>),
    Character(Option<Data>, Vec<Character>),
    Integer(Option<Data>, Vec<i64>),
    Float(Option<Data>, Vec<f64>),
}

impl Piece {

    fn validate_list(part: &Piece, seperator: &Option<Piece>, variant_registry: &VariantRegistry, templates: &Templates) -> Status<()> {
        confirm!(part.validate(variant_registry, templates));
        match seperator {
            Some(seperator) => confirm!(seperator.validate(variant_registry, templates)),
            None => ensure!(!part.calculate_widthless(templates).unwrap(), Message, string!("part may not be empty without a seperator")),
        }
        return success!(());
    }

    fn get_key(piece_stack: &mut DataStack, listed: bool, expected: bool) -> Status<Option<Data>> {
        if let Some(next) = piece_stack.peek(0) {
            if next.is_key() {
                ensure!(!listed, Message, string!("parts and seperators may not have a key"));
                piece_stack.advance(1);
                return success!(Some(next));
            }
        }
        ensure!(!expected, Message, string!("expected key"));
        return success!(None);
    }

    fn template_filters(piece_stack: &mut DataStack, direct_dependencies: &mut Vec<Data>) -> Status<Vec<Data>> {
        let mut filters = Vec::new();
        if let Some(next) = piece_stack.pop() {
            for filter in unpack_list!(&next).iter().cloned() {
                if !direct_dependencies.contains(&filter) {
                    direct_dependencies.push(filter.clone());
                }
                filters.push(filter);
            }
        }
        // TODO: ENSURE FILTERS IN NOT EMPTY
        return success!(filters);
    }

    pub fn parse(piece_source: &Data, direct_dependencies: &mut Vec<Data>, listed: bool) -> Status<Piece> {
        let piece_list = unpack_list!(piece_source);
        let mut piece_stack = DataStack::new(&piece_list);

        let piece_type = expect!(piece_stack.pop(), Message, string!("expected piece type"));
        match unpack_keyword!(&piece_type).printable().as_str() {

            "list" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let part_source = expect!(piece_stack.pop(), Message, string!("expected part"));
                let part = confirm!(Piece::parse(&part_source, direct_dependencies, true));
                let seperator = match piece_stack.pop() {
                    Some(seperator_source) => Some(confirm!(Piece::parse(&seperator_source, direct_dependencies, true))),
                    None => None,
                };
                confirm!(piece_stack.ensure_empty(), Tag, string!("list"));
                return success!(Piece::List(key, Box::new(part), Box::new(seperator)));
            }

            "confirmed" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let part_source = expect!(piece_stack.pop(), Message, string!("expected part"));
                let part = confirm!(Piece::parse(&part_source, direct_dependencies, true));
                let seperator = match piece_stack.pop() {
                    Some(seperator_source) => Some(confirm!(Piece::parse(&seperator_source, direct_dependencies, true))),
                    None => None, // PANIC ON THIS
                };
                confirm!(piece_stack.ensure_empty(), Tag, string!("confirmed"));
                return success!(Piece::Confirmed(key, Box::new(part), Box::new(seperator)));
            }

            "template" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = confirm!(Piece::template_filters(&mut piece_stack, direct_dependencies));
                ensure!(!filters.is_empty(), Message, string!("templates must have a filter"));
                confirm!(piece_stack.ensure_empty(), Tag, string!("template"));
                return success!(Piece::Template(key, filters));
            }

            "merge" => {
                ensure!(!listed, Message, string!("merge may not be used in a list"));
                let filters = confirm!(Piece::template_filters(&mut piece_stack, direct_dependencies));
                ensure!(!filters.is_empty(), Message, string!("templates must have a filter"));
                confirm!(piece_stack.ensure_empty(), Tag, string!("merge"));
                return success!(Piece::Merge(filters));
            }

            "data" => {
                ensure!(!listed, Message, string!("data may not be used in a list"));
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, true)).unwrap();
                let immediate = expect!(piece_stack.pop(), Message, string!("expected immediate"));
                confirm!(piece_stack.ensure_empty(), Tag, string!("data"));
                return success!(Piece::Data(key, immediate));
            }

            "comment" => {
                ensure!(!listed, Message, string!("comment may not be used in a list"));
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, true)).unwrap();
                confirm!(piece_stack.ensure_empty(), Tag, string!("comment"));
                return success!(Piece::Comment(key));
            }

            "keyword" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_identifier);
                confirm!(piece_stack.ensure_empty(), Tag, string!("keyword"));
                return success!(Piece::Keyword(key, filters));
            }

            "operator" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_identifier);
                confirm!(piece_stack.ensure_empty(), Tag, string!("operator"));
                return success!(Piece::Operator(key, filters));
            }

            "identifier" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_identifier);
                confirm!(piece_stack.ensure_empty(), Tag, string!("identifier"));
                return success!(Piece::Identifier(key, filters));
            }

            "type_identifier" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_identifier);
                confirm!(piece_stack.ensure_empty(), Tag, string!("type_identifier"));
                return success!(Piece::TypeIdentifier(key, filters));
            }

            "string" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_string);
                confirm!(piece_stack.ensure_empty(), Tag, string!("string"));
                return success!(Piece::String(key, filters));
            }

            "character" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_character);
                confirm!(piece_stack.ensure_empty(), Tag, string!("character"));
                return success!(Piece::Character(key, filters));
            }

            "integer" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_integer);
                confirm!(piece_stack.ensure_empty(), Tag, string!("integer"));
                return success!(Piece::Integer(key, filters));
            }

            "float" => {
                let key = confirm!(Piece::get_key(&mut piece_stack, listed, false));
                let filters = filters!(&mut piece_stack, unpack_float);
                confirm!(piece_stack.ensure_empty(), Tag, string!("float"));
                return success!(Piece::Float(key, filters));
            }

            invalid => return error!(Message, string!("invalid template piece {}", invalid)),
        };
    }

    fn filter_widthless(filters: &Vec<Data>, templates: &Templates) -> Option<bool> {
        let mut widthless = Some(false);
        if filters.is_empty() {
            for template in templates.values() {
                if let Some(widthless) = template.widthless {
                    if widthless {
                        return Some(true);
                    }
                } else {
                    widthless = None;
                }
            }
        } else {
            for filter in filters.iter() {
                let template = templates.get(filter).unwrap();
                if let Some(widthless) = template.widthless {
                    if widthless {
                        return Some(true);
                    }
                } else {
                    widthless = None;
                }
            }
        }
        return widthless;
    }

    pub fn calculate_widthless(&self, templates: &Templates) -> Option<bool> {
        match self {
            Piece::List(_, part, _) => return part.calculate_widthless(templates),
            Piece::Template(_, filters) => return Piece::filter_widthless(filters, templates),
            Piece::Merge(filters) => return Piece::filter_widthless(filters, templates),
            Piece::Comment(..) => return Some(true),
            Piece::Data(..) => return Some(true),
            _piece => return Some(false),
        }
    }

    pub fn validate(&self, variant_registry: &VariantRegistry, templates: &Templates) -> Status<()> {
        match self {
            Piece::Data(..) => return success!(()),
            Piece::Comment(..) => return success!(()),
            Piece::List(_, part, seperator) => return Piece::validate_list(part, seperator, variant_registry, templates),
            Piece::Confirmed(_, part, seperator) => return Piece::validate_list(part, seperator, variant_registry, templates),
            Piece::Template(..) => return success!(()),
            Piece::Merge(..) => return success!(()),
            Piece::Keyword(_, filters) => return variant_registry.validate_keywords(filters),
            Piece::Operator(_, filters) => return variant_registry.validate_operators(filters),
            Piece::Identifier(_, filters) => return variant_registry.validate_identifiers(filters),
            Piece::TypeIdentifier(_, filters) => return variant_registry.validate_type_identifiers(filters),
            Piece::String(..) => return variant_registry.validate_strings(),
            Piece::Character(..) => return variant_registry.validate_characters(),
            Piece::Integer(_, filters) => return variant_registry.validate_integers(filters),
            Piece::Float(_, filters) => return variant_registry.validate_floats(filters),
        }
    }

    fn add_token_list(token_list: &mut Vec<Data>, from: &str) -> bool {
        let location = identifier!(from);
        if !token_list.contains(&location) {
            token_list.push(location);
        }
        return true;
    }

    fn add_template_list(template_list: &mut Vec<Data>, filters: &Vec<Data>, templates: &Templates) -> bool {
        let mut widthless = false;
        if filters.is_empty() {
            for location in templates.keys() {
                widthless |= templates.get(location).unwrap().widthless.unwrap();
                if !template_list.contains(location) {
                    template_list.push(location.clone());
                }
            }
        } else {
            for location in filters.iter() {
                widthless |= templates.get(location).unwrap().widthless.unwrap();
                if !template_list.contains(location) {
                    template_list.push(location.clone());
                }
            }
        }
        return !widthless;
    }

    fn add_list_list(confirmed: bool, part: &Piece, seperator: &Option<Piece>, token_list: &mut Vec<Data>, template_list: &mut Vec<Data>, variant_registry: &VariantRegistry, templates: &Templates) -> bool {
        if !part.generate_start_list(token_list, template_list, variant_registry, templates) {
            if let Some(seperator) = seperator {
                seperator.generate_start_list(token_list, template_list, variant_registry, templates);
            }
            return confirmed;
        }
        return true;
    }

    pub fn generate_start_list(&self, token_list: &mut Vec<Data>, template_list: &mut Vec<Data>, variant_registry: &VariantRegistry, templates: &Templates) -> bool {
        match self {
            Piece::Data(..) => return false,
            Piece::Comment(..) => return false,
            Piece::Template(_, filters) => return Piece::add_template_list(template_list, filters, templates),
            Piece::Merge(filters) => return Piece::add_template_list(template_list, filters, templates),
            Piece::List(_, part, seperator) => return Piece::add_list_list(false, part, seperator, token_list, template_list, variant_registry, templates),
            Piece::Confirmed(_, part, seperator) => return Piece::add_list_list(true, part, seperator, token_list, template_list, variant_registry, templates),
            Piece::Keyword(_, filters) => return typed_token_list!(Keyword, token_list, filters, variant_registry),
            Piece::Operator(_, filters) => return typed_token_list!(Operator, token_list, filters, variant_registry),
            Piece::Identifier(..) => return Piece::add_token_list(token_list, "identifier"),
            Piece::TypeIdentifier(..) => return Piece::add_token_list(token_list, "type_identifier"),
            Piece::String(..) => return Piece::add_token_list(token_list, "string"),
            Piece::Character(..) => return Piece::add_token_list(token_list, "character"),
            Piece::Integer(..) => return Piece::add_token_list(token_list, "integer"),
            Piece::Float(..) => return Piece::add_token_list(token_list, "float"),
        }
    }

    fn create_widthless_filter(filters: &Vec<Data>, decisions: &mut SharedVector<Decision>, templates: &Templates) {
        for filter in filters.iter() {
            let template = templates.get(filter).unwrap();
            if let Some(widthless) = template.widthless {
                if widthless {
                    decisions.push(Decision::Template(filter.clone()));
                    template.create_widthless(decisions, templates);
                    return;
                }
            }
        }
    }

    pub fn create_widthless(&self, decisions: &mut SharedVector<Decision>, templates: &Templates) {
        match self {
            Piece::Data(..) => return,
            Piece::Comment(..) => return,
            Piece::Template(_, filters) => return Piece::create_widthless_filter(filters, decisions, templates),
            Piece::Merge(filters) => return Piece::create_widthless_filter(filters, decisions, templates),
            Piece::List(_, part, _) => part.create_widthless(decisions, templates),
            Piece::Confirmed(..) => panic!("this piece no widthless"),
            Piece::Keyword(..) => panic!("this piece no widthless"),
            Piece::Operator(..) => panic!("this piece no widthless"),
            Piece::Identifier(..) => panic!("this piece no widthless"),
            Piece::TypeIdentifier(..) => panic!("this piece no widthless"),
            Piece::String(..) => panic!("this piece no widthless"),
            Piece::Character(..) => panic!("this piece no widthless"),
            Piece::Integer(..) => panic!("this piece no widthless"),
            Piece::Float(..) => panic!("this piece no widthless"),
        }
    }
}
