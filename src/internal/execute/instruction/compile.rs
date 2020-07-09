use internal::*;
use std::path::Path;
use tokenize::tokenize;
use parse::parse;
use build::build;

fn print_stage(prefix: &str, stage: &str, compiler: &Data, build: &Data, context: &Data) -> Status<()> {
    let formatter_function_path = path!(vector![keyword!(str, "function"), keyword!(str, prefix)]);
    match confirm!(compiler.index(&formatter_function_path)) {
        Some(formatter_function) => { confirm!(function(&formatter_function, Vector::new(), &None, &compiler, build, context)); },
        None => println!("{}...", stage),
    }
    return success!(());
}

fn find_source_file(compiler: &Data, source_directory: &mut VectorString, module_name: &VectorString) -> Status<VectorString> {

    let file_settings = index_field!(compiler, "file_settings");
    ensure!(file_settings.is_map(), ExpectedFound, expected_list!["map"], file_settings.clone());

    let extention = expect!(confirm!(file_settings.index(&keyword!(str, "extention"))), MissingEntry, keyword!(str, "extention"));
    let extention = unpack_string!(&extention);

    if let Some(submodule_file) = confirm!(file_settings.index(&keyword!(str, "submodule"))) {
        let submodule_source_file = format_vector!("{}{}/{}.{}", source_directory, module_name, unpack_string!(&submodule_file), extention);
        if Path::new(&submodule_source_file.printable()).exists() {
            source_directory.push_str(module_name);
            source_directory.push(Character::from_char('/'));
            return success!(submodule_source_file);
        }
    }

    let source_file = format_vector!("{}{}.{}", source_directory, module_name, extention);
    ensure!(Path::new(&source_file.printable()).exists(), MissingFile, string!(source_file));
    return success!(source_file);
}

fn compile(mut context: Data, compiler: Data, parents: Data, source_string: VectorString, source_file: Option<VectorString>, source_directory: VectorString) -> Status<Data> {

    let build_map = map!();

    let source_file_data = match &source_file {
        Some(file) => string!(file.clone()),
        None => keyword!(str, "none"),
    };

    confirm!(context.set_entry(&keyword!(str, "parents"), parents, true));
    confirm!(context.set_entry(&keyword!(str, "parameters"), list!(), true));
    confirm!(context.set_entry(&keyword!(str, "code"), string!(source_string.clone()), true));
    confirm!(context.set_entry(&keyword!(str, "file"), source_file_data, true));
    confirm!(context.set_entry(&keyword!(str, "directory"), string!(source_directory), true));

    confirm!(print_stage("tokenize", "tokenizing", &compiler, &build_map, &context));
    let (token_stream, variant_registry) = confirm!(tokenize(&compiler, source_string, source_file, &None, &build_map, &context, false));

    confirm!(print_stage("parse", "parsing", &compiler, &build_map, &context));
    let top = confirm!(parse(&compiler, &variant_registry, &token_stream));

    confirm!(print_stage("build", "building", &compiler, &build_map, &context));
    if let Some(build_map) = confirm!(build(&compiler, &top, build_map, &context)) {
        return success!(build_map);
    }

    return success!(top);
}

pub fn compile_module(parameters: Vec<Data>, context: &Data) -> Status<Data> {

    let mut iterator = parameters.iter();
    let compiler = iterator.next().unwrap().clone();
    let module_name = iterator.next().unwrap();
    let module_string = unpack_identifier!(module_name);

    let mut source_directory = match iterator.next().cloned() {
        Some(directory) => unpack_string!(&directory),
        None => {
            match confirm!(context.index(&keyword!(str, "directory"))) {
                Some(directory) => unpack_string!(&directory), // TODO: proper message
                None => VectorString::new(),
            }
        }
    };

    let source_file = confirm!(find_source_file(&compiler, &mut source_directory, &module_string));
    let source_string = confirm!(read_file(&source_file));

    let mut parent_list = match confirm!(context.index(&keyword!(str, "parents"))) {
        Some(parents) => unpack_list!(&parents), // TODO: proper message
        None => Vector::new(),
    };
    parent_list.push(identifier!(module_string));

    return compile(context.clone(), compiler, list!(parent_list), source_string, Some(source_file), source_directory);
}

pub fn compile_file(parameters: Vec<Data>, context: &Data) -> Status<Data> {

    let mut iterator = parameters.iter();
    let compiler = iterator.next().unwrap().clone();
    let source_file = iterator.next().cloned().unwrap();
    let source_file = unpack_string!(&source_file);
    let source_string = confirm!(read_file(&source_file));

    let source_directory = match confirm!(context.index(&keyword!(str, "directory"))) {
        Some(directory) => unpack_string!(&directory),
        None => VectorString::new(),
    };

    let parents = match confirm!(context.index(&keyword!(str, "parents"))) {
        Some(parents) => parents,
        None => list!(),
    };
    unpack_list!(&parents); // TODO: proper message

    return compile(context.clone(), compiler, parents, source_string, Some(source_file), source_directory);
}

pub fn compile_string(parameters: Vec<Data>, context: &Data) -> Status<Data> {

    let mut iterator = parameters.iter();
    let compiler = iterator.next().unwrap().clone();
    let source_string = iterator.next().cloned().unwrap();
    let source_string = unpack_string!(&source_string);

    let source_directory = match confirm!(context.index(&keyword!(str, "directory"))) {
        Some(directory) => unpack_string!(&directory),
        None => VectorString::new(),
    };

    let parents = match confirm!(context.index(&keyword!(str, "parents"))) {
        Some(parents) => parents,
        None => list!(),
    };
    unpack_list!(&parents); // TODO: proper message

    return compile(context.clone(), compiler, parents, source_string, None, source_directory);
}

pub fn tokenize_string(_compiler: &Data, _source: &Data, _context: &Data) -> Status<Data> {
    //let token_stream = confirm!(tokenize(&compiler, unpack_string!(source), None, context)).0;
    // serialize token strem
    return success!(boolean!(true));
}

pub fn parse_token_stream(_compiler: &Data, _source: &Data, _context: &Data) -> Status<Data> {
    //let token_stream = ???;
    //let variant_registry = ???;
    //let top = confirm!(parse(&compiler, variant_registry, token_stream, context));
    return success!(boolean!(true));
}

pub fn build_top(compiler: &Data, top: &Data, context: &Data) -> Status<Data> {
    match confirm!(build(compiler, top, map!(), context)) {
        Some(build_map) => return success!(build_map),
        None => return success!(top.clone()),
    }
}
