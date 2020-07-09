use internal::*;

pub fn build(compiler: &Data, top: &Data, build: Data, context: &Data) -> Status<Option<Data>> {

    if let Some(pipeline) = confirm!(compiler.index(&keyword!(str, "pipeline"))) {
        let pipeline_list = unpack_list!(pipeline, Message, string!(str, "pipeline needs to be a list"));
        let mut new_top = top.clone(); // TODO: transfer for optimization

        for pass in pipeline_list.iter() {
            let pass_name = unpack_identifier!(pass, Message, string!(str, "pass must be an identifier"));
            //let new_context = confirm!(context.overwrite(&keyword!(str, "pass"), pass.clone()));
            new_top = confirm!(new_top.pass(&Some(pass_name), compiler, &build, context));
        }
        return success!(Some(build));
    }

    return success!(None);
}
