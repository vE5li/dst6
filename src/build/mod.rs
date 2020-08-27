use internal::*;
use debug::*;

pub fn build(compiler: &Data, top: &Data) -> Status<Data> {
    let build = map!();

    if let Some(pipeline) = confirm!(compiler.index(&keyword!("pipeline"))) {
        let pipeline_list = unpack_list!(pipeline, string!("pipeline needs to be a list"));
        let mut new_top = top.clone(); // TODO: transfer for optimization

        for name in pipeline_list.into_iter() {
            ensure!(name.is_literal(), string!("pass name must be a literal"));
            let pass = Pass::new(name, Vec::new());
            new_top = confirm!(new_top.pass(&pass, compiler, &build));
        }
    }

    return success!(build);
}

pub fn call_build(compiler: &Data, top: &Data) -> Status<Data> {
    return build(compiler, top);
}
