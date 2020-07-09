use internal::*;

#[derive(Clone, PartialEq, Eq)]
enum PositionKey {
    Some(Option<VectorString>, VectorString),
    None,
}

impl Compare for PositionKey {

    fn compare(&self, other: &Self) -> Relation {
        if let PositionKey::Some(file, source) = self {
            if let PositionKey::Some(other_file, other_source) = other {
                if let Some(file) = file {
                    if let Some(other_file) = other_file {
                        match file.compare(other_file) {
                            Relation::Equal => {},
                            relation => return relation,
                        }
                    }
                }
                return source.compare(other_source);
            }
        }
        panic!();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub file:       Option<VectorString>,
    pub source:     VectorString,
    pub line:       usize,
    pub character:  usize,
    pub length:     usize,
}

impl Position {

    pub fn new(file: Option<VectorString>, source: VectorString, line: usize, character: usize, length: usize) -> Position {
        Self {
            file:       file,
            source:     source,
            line:       line,
            character:  character,
            length:     length,
        }
    }

    pub fn parse_positions(formatted_positions: &Data) -> Status<Vec<Position>> {
        let mut positions = Vec::new();
        for item in unpack_list!(formatted_positions).iter() {
            let file = confirm!(item.index(&identifier!(str, "file")));
            let file = expect!(file, Message, string!(str, "position may not miss the file field"));
            let source = confirm!(item.index(&identifier!(str, "source")));
            let source = expect!(source, Message, string!(str, "position may not miss the source field"));
            let line = confirm!(item.index(&identifier!(str, "line")));
            let line = expect!(line, Message, string!(str, "position may not miss the line field"));
            let character = confirm!(item.index(&identifier!(str, "character")));
            let character = expect!(character, Message, string!(str, "position may not miss the character field"));
            let length = confirm!(item.index(&identifier!(str, "length")));
            let length = expect!(length, Message, string!(str, "position may not miss the length field"));

            let file = match file == identifier!(str, "none") {
                true => None,
                false => Some(unpack_string!(&file)),
            };
            let source = unpack_string!(&source);
            let line = unpack_integer!(&line) as usize;
            let character = unpack_integer!(&character) as usize;
            let length = unpack_integer!(&length) as usize;
            positions.push(Position::new(file, source, line, character, length));
        }
        return success!(positions);
    }

    pub fn serialize_positions(positions: &Vec<Position>) -> Data {
        let mut items = Vector::new();
        for position in positions {
            let mut map = Map::new();
            match &position.file {
                Some(file) => map.insert(identifier!(str, "file"), string!(file.clone())),
                None => map.insert(identifier!(str, "file"), identifier!(str, "none")),
            };
            map.insert(identifier!(str, "source"), string!(position.source.clone()));
            map.insert(identifier!(str, "line"), integer!(position.line as i64));
            map.insert(identifier!(str, "character"), integer!(position.character as i64));
            map.insert(identifier!(str, "length"), integer!(position.length as i64));
            items.push(map!(map));
        }
        return list!(items);
    }

    fn insert_positions(positions_map: &mut Map<PositionKey, Vec<Position>>, positions: Vec<Position>) {
        for iterator in positions.into_iter() {
            let key = PositionKey::Some(iterator.file.clone(), iterator.source.clone());
            if let Some(map_entry) = positions_map.get_mut(&key) {
                if !map_entry.contains(&iterator) {
                    map_entry.push(iterator);
                }
                continue;
            }
            positions_map.insert(key, vec![iterator]);
        }
    }

    fn sort_positions(positions_map: &mut Map<PositionKey, Vec<Position>>) {
        // TODO: ENSURE length IS NOT 0!!!!!!!!!!!!!!!!!!!1

        let sorter = |left: &Position, right: &Position| {
            if left.line != right.line {
                return left.line.cmp(&right.line);
            };
            return left.character.cmp(&right.character);
        };

        for positions in positions_map.values_mut() {
            positions.sort_by(sorter);
        }
    }

    pub fn fuze(positions: Vec<Position>, internal: bool) -> Vec<Position> {
        let mut positions_map = Map::new();
        let mut return_positions = Vec::new();

        if internal {
            positions_map.insert(PositionKey::None, positions);
        } else {
            Position::insert_positions(&mut positions_map, positions);
            Position::sort_positions(&mut positions_map);
        }

        for mut positions in positions_map.values().cloned() {
            if positions.is_empty() {
                continue;
            }

            let mut offset = 0;
            while offset < positions.len() - 1 {
                if positions[offset].line == positions[offset + 1].line && positions[offset].character + positions[offset].length == positions[offset + 1].character {
                    positions[offset].length += positions[offset + 1].length;
                    positions.remove(offset + 1);
                } else {
                    offset += 1;
                }
            }

            return_positions.append(&mut positions);
        }

        return return_positions;
    }

    pub fn range(positions: Vec<Position>, internal: bool) -> Vec<Position> {
        let mut positions_map = Map::new();
        let mut return_positions = Vec::new();

        if internal {
            positions_map.insert(PositionKey::None, positions);
        } else {
            Position::insert_positions(&mut positions_map, positions);
            Position::sort_positions(&mut positions_map);
        }

        for positions in positions_map.values() {
            if positions.is_empty() {
                continue;
            }

            if positions.len() == 1 {
                return_positions.push(positions[0].clone());
                continue;
            }

            let first = positions[0].clone();
            let last = positions.last().unwrap().clone();

            let mut offset = 0;
            let mut line = 1;
            let mut character = 1;
            while line != first.line || character != first.character + first.length {
                match first.source[offset].as_char() {

                    '\n' => {
                        character = 1;
                        line += 1;
                    }

                    _other => {
                        character += 1;
                    }
                }
                offset += 1;
            }

            return_positions.push(first);

            while line != last.line || character != last.character {
                match last.source[offset].as_char() {

                    '\n' => {
                        line += 1;
                        character = 1;
                        return_positions.push(Position::new(last.file.clone(), last.source.clone(), line, 1, 0));
                    }

                    _other => {
                        character += 1;
                        return_positions.last_mut().unwrap().length += 1;
                    }
                }
                offset += 1;
            }

            return_positions.last_mut().unwrap().length += last.length;
        }

        return_positions.retain(|position| position.length != 0);
        return return_positions;
    }
}
