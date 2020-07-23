mod template;
mod result;

use internal::*;
use debug::*;

pub use self::template::*;
pub use self::result::*;

pub type Templates = Map<Data, Template>;
pub type Processed = Vec<Map<Data, MatchResult>>;
pub type Dependencies = Map<Data, Vec<Data>>;
pub type Pool = Map<Data, Dependencies>;

macro_rules! template_matches_piece {
    ($template:expr, $paths:expr, $filters:expr, $parser:expr) => ({

        let mut paths = match $filters.iter().position(| filter | filter == $template) {
            Some(_position) => $paths.clone(),
            None => Vector::new(),
        };

        for (filter_index, filter) in $filters.iter().enumerate() {
            let template = $parser.templates.get(filter).unwrap();

            if let Some(widthless) = template.widthless {
                if widthless {
                    let mut decisions = vector![Decision::Filter(filter_index), Decision::Template(filter.clone())];
                    template.create_widthless(&mut decisions, &$parser.templates);
                    paths.push(Path::new(decisions, $paths[0].index, 0, false, None));
                }
            }
        }

        MatchResult::from(paths)
    });
}

macro_rules! token_matches_piece {
    ($type:ident, $token_stream:expr, $index:expr, $filters:expr) => ({
        if $token_stream.len() > $index {
            if let TokenType::$type(data) = &$token_stream[$index].token_type {
                if $filters.is_empty() {
                    let path = Path::new(Vector::new(), $index, 1, true, None);
                    return MatchResult::Matched(vector![path]);
                } else {
                    if let Some(filter_index) = $filters.iter().position(|filter| filter == data) {
                        let path = Path::new(vector![Decision::Filter(filter_index)], $index, 1, true, None);
                        return MatchResult::Matched(vector![path]);
                    }
                }
            }
        }
        MatchResult::Missed
    });
}

pub fn parse(compiler: &Data, token_stream: &Vec<Token>, variant_registry: &VariantRegistry) -> Status<Data> {
    let parseable_token_stream = token_stream.iter().filter(|token| token.parsable()).cloned().collect();
    let parser = confirm!(Parser::new(compiler, variant_registry, &parseable_token_stream));
    let (decision_stream, templates) = confirm!(parser.parse());

    let mut template_builder = TemplateBuilder::new(token_stream, &decision_stream, &templates);
    let (raw_module, _positions) = confirm!(template_builder.build(true));
    return success!(raw_module);
}

pub fn call_parse(compiler: &Data, token_stream: &Data, variant_registry: &Data) -> Status<Data> {
    let variant_registry = confirm!(VariantRegistry::deserialize(variant_registry));
    let token_stream = confirm!(deserialize_token_stream(token_stream));
    return parse(compiler, &token_stream, &variant_registry);
}

pub fn deserialize_token_stream(serialized: &Data) -> Status<Vec<Token>> {

    let file = confirm!(serialized.index(&identifier!("file")));
    let file = expect!(file, Message, string!("token stream may not miss the file field"));
    let file = (file != identifier!("none")).then_some(unpack_string!(&file));

    let source = confirm!(serialized.index(&identifier!("source")));
    let source = expect!(source, Message, string!("token stream may not miss the source field"));
    let source = unpack_string!(&source);

    let tokens = confirm!(serialized.index(&identifier!("tokens")));
    let tokens = expect!(tokens, Message, string!("token stream may not miss the tokens field"));

    let mut token_stream = Vec::new();
    for token in unpack_list!(&tokens).iter() {
        let token = confirm!(Token::deserialize(token, &file, &source));
        token_stream.push(token);
    }

    return success!(token_stream);
}

struct Parser<'p> {
    token_stream:       &'p Vec<Token>,
    templates:          Templates,
    template_pool:      Pool,
    token_pool:         Pool,
}

impl<'p> Parser<'p> {

    pub fn new(compiler: &Data, variant_registry: &VariantRegistry, token_stream: &'p Vec<Token>) -> Status<Self> {
        let mut templates = Map::new();
        let mut template_pool = Pool::new();
        let mut token_pool = Pool::new();

        let path = keyword!("template");
        let template_root = index!(compiler, &path);
        let mut dependencies = Dependencies::new();

        confirm!(Template::pull(&keyword!("top"), &mut templates, &mut dependencies, &template_root));
        let base_token_pool = Parser::create_base_token_pool(variant_registry);
        let base_template_pool = Parser::create_base_template_pool(&templates);

        let mut changed = true;
        while changed {
            let cloned = templates.clone();
            changed = false;

            for (location, _template) in cloned.iter() {
                let template = templates.get_mut(location).unwrap();
                changed |= template.calculate_widthless(&cloned);
            }
        }

        let cloned = templates.clone();
        for (location, template) in templates.iter_mut() {
            for flavor in template.flavors.iter_mut() {
                ensure!(flavor.calculate_widthless(&cloned).is_some(), Message, string!("failed to calculate falvor of {}", location.serialize())); // TODO: THIS MEANS LOOPED DEPENDENCY
            }
            confirm!(template.validate(variant_registry, &cloned));
            template.generate_start_list(variant_registry, &cloned);
        }

        for (location, template) in templates.iter() {
            let mut new_token_pool = base_token_pool.clone();
            let mut new_template_pool = base_template_pool.clone();

            Parser::collect_pools(location, template, &mut new_token_pool, &mut new_template_pool, &templates);
            token_pool.insert(location.clone(), new_token_pool);
            template_pool.insert(location.clone(), new_template_pool);
        }

        return success!(Self {
            token_stream:       token_stream,
            templates:          templates,
            template_pool:      template_pool,
            token_pool:         token_pool,
        })
    }

    fn create_base_token_pool(variant_registry: &VariantRegistry) -> Dependencies {

        let mut dependencies = Dependencies::new();

        for operator in variant_registry.avalible_operators().iter() {
            dependencies.insert(Data::Identifier(format_vector!("operator:{}", operator)), Vec::new());
        }

        for keyword in variant_registry.avalible_keywords().iter() {
            dependencies.insert(Data::Identifier(format_vector!("keyword:{}", keyword)), Vec::new());
        }

        if variant_registry.has_identifiers() {
            dependencies.insert(identifier!("identifier"), Vec::new());
        }

        if variant_registry.has_type_identifiers() {
            dependencies.insert(identifier!("type_identifier"), Vec::new());
        }

        if variant_registry.has_characters {
            dependencies.insert(identifier!("character"), Vec::new());
        }

        if variant_registry.has_strings {
            dependencies.insert(identifier!("string"), Vec::new());
        }

        if variant_registry.has_integers {
            dependencies.insert(identifier!("integer"), Vec::new());
        }

        if variant_registry.has_floats {
            dependencies.insert(identifier!("float"), Vec::new());
        }

        return dependencies;
    }

    fn create_base_template_pool(templates: &Templates) -> Dependencies {
        let mut dependencies = Dependencies::new();
        for location in templates.keys() {
            dependencies.insert(location.clone(), Vec::new());
        }
        return dependencies;
    }

    pub fn collect_pools(location: &Data, template: &Template, token_pool: &mut Dependencies, template_pool: &mut Dependencies, templates: &Templates) {

        if let Some(ref list) = template.token_list {
            for dependency in list.iter() {
                token_pool.get_mut(dependency).unwrap().push(location.clone());
            }
        }

        if let Some(ref list) = template.template_list {
            for dependency in list.iter() {
                template_pool.get_mut(dependency).unwrap().push(location.clone());
                let dependent_template = templates.get(dependency).unwrap();

                if let Some(ref list) = dependent_template.token_list {
                    if !list.is_empty() { // combine these
                        if !token_pool.get(&list[0]).unwrap().contains(dependency) {
                            Parser::collect_pools(dependency, dependent_template, token_pool, template_pool, templates);
                            continue;
                        }
                    }
                }

                if let Some(ref list) = dependent_template.template_list {
                    if !list.is_empty() { // combine these
                        if !template_pool.get(&list[0]).unwrap().contains(dependency) {
                            Parser::collect_pools(dependency, dependent_template, token_pool, template_pool, templates);
                            continue;
                        }
                    }
                }
            }
        }
    }

    fn derive(path: &Path, new_paths: &Vector<Path>) -> Vector<Path> {
        let mut derived_paths = Vector::new();

        for new_path in new_paths.iter() {
            let mut combined_decisions = path.decisions.clone();
            combined_decisions.append(&new_path.decisions);
            let combined_path = Path::new(combined_decisions, path.index, path.width + new_path.width, path.confirmed || new_path.confirmed, new_path.expected.clone());
            derived_paths.push(combined_path);
        }

        return derived_paths;
    }

    fn push_decision(paths: &mut Vector<Path>, decision: Decision) {
        for path in paths.iter_mut() {
            path.decisions.push(decision.clone());
        }
    }

    fn inject_decision(paths: &mut Vector<Path>, decision: Decision) {
        for path in paths.iter_mut() {
            path.decisions.insert(0, decision.clone());
        }
    }

    fn match_piece(&self, piece: &Piece, path: &Path, leading: &Option<(&Data, &Vector<Path>)>, processed: &mut Processed) -> MatchResult {
        if !path.confirmed {
            if let Some((leading_template, leading_paths)) = leading {
                return self.match_piece_from_template(piece, leading_template, leading_paths, processed);
            }
        }
        return self.match_piece_from_token(piece, path.confirmed, path.index + path.width, processed);
    }

    fn find(&self, destination: &Data, pool: &Vec<Data>, index: usize, leading: Option<(&Data, &Vector<Path>)>, found_paths: &mut Vector<Path>, processed: &mut Processed) {

        for location in pool.iter() {
            let template = self.templates.get(location).unwrap();
            let mut location_paths = Vector::new();

            'flavor: for (flavor_index, flavor) in template.flavors.iter().enumerate() {
                let mut active_paths = vector![Path::new(Vector::new(), index, 0, false, None)];

                'piece: for piece in flavor.pieces.iter() {
                    match piece {
                        Piece::Data(..) => continue 'piece,
                        Piece::Comment(..) => continue 'piece,
                        _piece => {},
                    }

                    for path in active_paths.transfer().iter() {
                        let result = self.match_piece(&piece, path, &leading, processed);
                        if let MatchResult::Matched(new_paths) = result {
                            active_paths.append(&Parser::derive(&path, &new_paths));
                        }
                        // else -> update best match
                    }

                    if active_paths.is_empty() {
                        continue 'flavor;
                    }
                }

                active_paths.retain(|path| path.confirmed);
                if active_paths.is_empty() {
                    continue 'flavor;
                }

                Parser::inject_decision(&mut active_paths, Decision::Flavor(flavor_index));
                Parser::inject_decision(&mut active_paths, Decision::Template(location.clone()));
                location_paths.append(&active_paths);
                active_paths.clear();
            }

            if !location_paths.is_empty() {
                let result = self.paths_from_template(destination, location, &location_paths, processed);

                if location == destination {
                    found_paths.append(&location_paths);
                    location_paths.clear();
                }

                result.update(found_paths);
            }
        }

        Parser::reduce_paths(found_paths);

        if leading.is_none() {
            let result = self.create_widthless(destination, index);
            result.update(found_paths);
        }
    }

    pub fn reduce_paths(paths: &mut Vector<Path>) {
        let mut base = 0;

        'outer: while base + 1 < paths.len() {
            let mut offset = base + 1;

            while offset < paths.len() {
                match paths[base].evaluate(&paths[offset]) {

                    Some(true) => {
                        paths.remove(base);
                        continue 'outer;
                    }

                    Some(false) => { paths.remove(offset); },

                    None => offset += 1,
                }
            }
            base += 1;
        }
    }

    pub fn paths_from_token(&self, destination: &Data, index: usize, processed: &mut Processed) -> MatchResult {

        if let Some(result) = processed[index].get(destination) {
            return result.clone();
        }

        let mut found_paths = Vector::new();
        if self.token_stream.len() > index {
            let destination_pool = self.token_pool.get(destination).unwrap();
            let relevant_pool = destination_pool.get(&self.token_stream[index].to_location()).unwrap();
            self.find(destination, relevant_pool, index, None, &mut found_paths, processed);
        } else {
            let result = self.create_widthless(destination, index);
            result.update(&mut found_paths);
        }

        let result = MatchResult::from(found_paths);
        processed[index].insert(destination.clone(), result.clone());
        return result;
    }

    fn paths_from_template(&self, destination: &Data, leading_template: &Data, leading_paths: &Vector<Path>, processed: &mut Processed) -> MatchResult {
        let mut found_paths = Vector::new();
        let destination_pool = self.template_pool.get(destination).unwrap();
        let relevant_pool = destination_pool.get(leading_template).unwrap();

        self.find(destination, relevant_pool, leading_paths[0].index, Some((leading_template, leading_paths)), &mut found_paths, processed);
        return MatchResult::from(found_paths); //
    }

    fn match_piece_from_token(&self, piece: &Piece, follow: bool, index: usize, processed: &mut Processed) -> MatchResult {
        match piece {
            Piece::Data(..) => panic!("data may not be matched"),
            Piece::Comment(..) => panic!("comment may not be matched"),
            Piece::Template(_, filters) => return self.filtered_paths_from_token(filters, follow, index, processed),
            Piece::Merge(filters) => return self.filtered_paths_from_token(filters, follow, index, processed),
            Piece::List(_, part, seperator) => return self.list_from_token(part, seperator, false, follow, index, processed),
            Piece::Confirmed(_, part, seperator) => return self.list_from_token(part, seperator, true, follow, index, processed),
            Piece::Keyword(_, filters) => return token_matches_piece!(Keyword, &self.token_stream, index, filters),
            Piece::Operator(_, filters) => return token_matches_piece!(Operator, &self.token_stream, index, filters),
            Piece::Identifier(_, filters) => return token_matches_piece!(Identifier, &self.token_stream, index, filters),
            Piece::TypeIdentifier(_, filters) => return token_matches_piece!(TypeIdentifier, &self.token_stream, index, filters),
            Piece::String(_, filters) => return token_matches_piece!(String, &self.token_stream, index, filters),
            Piece::Character(_, filters) => return token_matches_piece!(Character, &self.token_stream, index, filters),
            Piece::Integer(_, filters) => return token_matches_piece!(Integer, &self.token_stream, index, filters),
            Piece::Float(_, filters) => return token_matches_piece!(Float, &self.token_stream, index, filters),
        }
    }

    fn match_piece_from_template(&self, piece: &Piece, leading_template: &Data, leading_paths: &Vector<Path>, processed: &mut Processed) -> MatchResult {
        match piece {
            Piece::Data(..) => panic!("data may not be matched"),
            Piece::Comment(..) => panic!("comment may not be matched"),
            Piece::Template(_, filters) => return template_matches_piece!(leading_template, leading_paths, filters, self),
            Piece::Merge(filters) => return template_matches_piece!(leading_template, leading_paths, filters, self),
            Piece::List(_, part, seperator) => return self.list_from_template(part, seperator, false, leading_template, leading_paths, processed),
            Piece::Confirmed(_, part, seperator) => return self.list_from_template(part, seperator, true, leading_template, leading_paths, processed),
            _piece => return MatchResult::Missed,
        }
    }

    fn active_paths_from_token(&self, piece: &Piece, active_paths: &mut Vector<Path>, processed: &mut Processed) {
        for path in active_paths.transfer().iter() {
            if let MatchResult::Matched(new_paths) = self.match_piece_from_token(piece, path.confirmed, path.index + path.width, processed) {
                let derived_paths = Parser::derive(&path, &new_paths);
                active_paths.append(&derived_paths);
            }
        }
    }

    fn active_paths_from_template(&self, piece: &Piece, leading_template: &Data, leading_paths: &Vector<Path>, active_paths: &mut Vector<Path>, processed: &mut Processed) {
        for path in active_paths.transfer().iter() {
            let result = match path.confirmed {
                true => self.match_piece_from_token(piece, true, path.index + path.width, processed),
                false => self.match_piece_from_template(piece, leading_template, leading_paths, processed),
            };

            if let MatchResult::Matched(new_paths) = result {
                let derived_paths = Parser::derive(&path, &new_paths);
                active_paths.append(&derived_paths);
            }
        }
    }

    fn filtered_paths_from_token(&self, filters: &Vec<Data>, follow: bool, index: usize, processed: &mut Processed) -> MatchResult {
        let mut paths = Vector::new();

        for filter in filters.iter() {
            match follow {
                true => self.paths_from_token(filter, index, processed).update(&mut paths),
                false => self.create_widthless(filter, index).update(&mut paths),
            }
        }

        return MatchResult::from(paths);
    }

    fn create_widthless(&self, location: &Data, index: usize) -> MatchResult {
        let template = self.templates.get(location).unwrap();

        if let Some(widthless) = template.widthless {
            if widthless {
                let mut decisions = vector![Decision::Template(location.clone())];
                template.create_widthless(&mut decisions, &self.templates);
                return MatchResult::Matched(vector![Path::new(decisions, index, 0, false, None)]); // none?
            }
        }

        return MatchResult::Missed;
    }

    fn list_from_token(&self, part: &Piece, seperator: &Option<Piece>, confirmed: bool, follow: bool, index: usize, processed: &mut Processed) -> MatchResult {
        let mut active_paths = vector![Path::new(Vector::new(), index, 0, follow, None)];
        let mut found_paths = Vector::new();
        let mut counter = 0;

        while !active_paths.is_empty() {
            self.active_paths_from_token(part, &mut active_paths, processed);

            if !confirmed || counter != 0 {
                for path in active_paths.iter() {
                    found_paths.push(path.clone());
                }
            }

            Parser::push_decision(&mut active_paths, Decision::Next);
            if let Some(seperator) = seperator {
                self.active_paths_from_token(seperator, &mut active_paths, processed);
            }
            counter += 1;
        }

        Parser::reduce_paths(&mut found_paths);
        Parser::inject_decision(&mut found_paths, Decision::List);
        Parser::push_decision(&mut found_paths, Decision::End);
        return MatchResult::from(found_paths); // part
    }

    fn list_from_template(&self, part: &Piece, seperator: &Option<Piece>, confirmed: bool, leading_template: &Data, leading_paths: &Vector<Path>, processed: &mut Processed) -> MatchResult {
        let mut active_paths = vector![Path::new(Vector::new(), leading_paths[0].index, 0, false, None)];
        let mut found_paths = Vector::new();
        let mut counter = 0;

        while !active_paths.is_empty() {
            self.active_paths_from_template(part, leading_template, leading_paths, &mut active_paths, processed);

            if !confirmed || counter != 0 {
                active_paths.iter().cloned().for_each(|path| found_paths.push(path));
            }

            Parser::push_decision(&mut active_paths, Decision::Next);

            if let Some(seperator) = seperator {
                self.active_paths_from_template(seperator, leading_template, leading_paths, &mut active_paths, processed);
            }
            counter += 1;
        };

        Parser::reduce_paths(&mut found_paths);
        Parser::inject_decision(&mut found_paths, Decision::List);
        Parser::push_decision(&mut found_paths, Decision::End);
        return MatchResult::from(found_paths); // part
    }

    fn decision_stream(&self, result: MatchResult) -> Status<Vector<Decision>> {
        if let MatchResult::Matched(paths) = result {
            if let Some(best) = paths.into_iter().find(|path| path.width == self.token_stream.len()) {
                return success!(best.decisions);
            }
        }
        // output error with best match
        return error!(Message, string!("failed to parse main"));
    }

    pub fn parse(self) -> Status<(Vector<Decision>, Templates)> {
        let mut processed: Vec<Map<Data, MatchResult>> = self.token_stream.iter().map(|_| Map::new()).collect();
        processed.push(Map::new());
        let result = self.paths_from_token(&keyword!("top"), 0, &mut processed);
        let decision_stream = confirm!(self.decision_stream(result));

        return success!((decision_stream, self.templates));
    }
}
