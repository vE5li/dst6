use internal::*;

use std::result::Result;
use std::io::prelude::*;
use std::fs::{ File, read_dir, metadata };

fn read_file_raw(path: &VectorString) -> Status<String> {
    let mut string = String::new();
    let mut file = match File::open(path.printable()) {
        Err(..) => return error!(Message, string!(str, "missing file \"{}\"", path)), // MissingFIle
        Ok(file) => file,
    };
    file.read_to_string(&mut string).unwrap(); // error handling
    return success!(string);
}

pub fn read_file(path: &VectorString) -> Status<VectorString> {
    return success!(VectorString::from(&confirm!(read_file_raw(path))));
}

pub fn read_map(path: &VectorString) -> Status<Data> {
    let mut string = confirm!(read_file_raw(path));
    string.insert(0, '{');
    string.push('}');
    let mut character_stack = CharacterStack::new(VectorString::from(&string), None);
    return parse_data(&mut character_stack);
}

pub fn read_list(path: &VectorString) -> Status<Data> {
    let mut string = confirm!(read_file_raw(path));
    string.insert(0, '[');
    string.push(']');
    let mut character_stack = CharacterStack::new(VectorString::from(&string), None);
    return parse_data(&mut character_stack);
}

fn write_file_raw(path: &VectorString, string: &str) -> Status<()> {
    let mut file = File::create(&path.printable()).unwrap(); // error handling
    write!(&mut file, "{}", string).unwrap(); // error handling
    return success!(());
}

pub fn write_file(path: &VectorString, string: &VectorString) -> Status<()> {
    return write_file_raw(path, &string.printable());
}

pub fn write_map(path: &VectorString, instance: &Data) -> Status<()> {
    match instance {
        Data::Map(map) => {
            let mut string = String::new();
            for (key, instance) in map.iter() {
                string.push_str(&format!("{} {}\n", key.serialize(), instance.serialize()));
            }
            return write_file_raw(path, &string);
        }
        _invalid => return error!(ExpectedFound, expected_list!["map"], instance.clone()),
    }
}

pub fn write_list(path: &VectorString, instance: &Data) -> Status<()> {
    match instance {
        Data::List(items) => {
            let mut string = String::new();
            for instance in items.iter() {
                string.push_str(&format!("{}\n", instance.serialize()));
            }
            return write_file_raw(path, &string);
        }
        invalid => return error!(ExpectedFound, expected_list!["list"], invalid.clone()),
    }
}

pub fn get_directory_entries(path: &VectorString) -> Status<Vec<VectorString>> {

    let paths = match read_dir(path.serialize()) {
        Result::Err(..) => return error!(Message, string!(str, "directory missing")),
        Result::Ok(paths) => paths,
    };

    let mut entries = Vec::new();
    for file in paths {
        let file = file.unwrap(); // TODO:
        let file_name = file.file_name().into_string().unwrap();
        let mut file_type = file.file_type().unwrap();

        if file_type.is_symlink() {
            let metadata = metadata(&format!("{}{}", path, file_name)).unwrap();
            file_type = metadata.file_type();
        }

        match file_type.is_file() {
            true => entries.push(VectorString::from(&file_name)),
            false => entries.push(format_vector!("{}/", file_name)),
        }
    }

    entries.sort_by(|entry, other| entry.compare(other).into_ordering());
    return success!(entries);
}
