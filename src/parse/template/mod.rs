mod piece;
mod flavor;
mod builder;

use internal::*;
use debug::*;

pub use self::piece::Piece;
pub use self::flavor::Flavor;
pub use self::builder::TemplateBuilder;

use parse::{ Decision, Templates, Dependencies };

#[derive(Debug, Clone)]
pub struct Template {
    pub passes:         Option<Data>,
    pub flavors:        Vec<Flavor>,
    pub widthless:      Option<bool>,
    pub token_list:     Option<Vec<Data>>,
    pub template_list:  Option<Vec<Data>>,
}

impl Template {

    pub fn new(flavors: Vec<Flavor>, passes: Option<Data>) -> Self {
        Self {
            flavors:        flavors,
            passes:         passes,
            widthless:      None,
            token_list:     None,
            template_list:  None,
        }
    }

    pub fn pull(location: &Data, templates: &mut Templates, dependencies: &mut Dependencies, template_root: &Data) -> Status<()> {
        let template_source = index!(template_root, location);
        let template_source = unpack_list!(&template_source);
        let mut template_stack = DataStack::new(&template_source);

        let mut direct_dependencies = Vec::new();
        let mut flavors = Vec::new();
        let mut passes = None;

        if let Some(first) = template_stack.peek(0) {
            if first.is_map() {
                template_stack.advance(1);
                passes = Some(first);
            }
        }

        while let Some(flavor) = template_stack.pop() {
            let mut pieces = Vec::new();
            for piece in unpack_list!(&flavor).into_iter() {
                let piece = confirm!(Piece::parse(&piece, &mut direct_dependencies, false));
                pieces.push(piece);
            }
            flavors.push(Flavor::new(pieces));
        }

        ensure!(!flavors.is_empty(), Message, string!("template {} does not have any flavors", location.serialize()));
        templates.insert(location.clone(), Template::new(flavors, passes));

        for dependency in &direct_dependencies {
            if templates.get(dependency).is_none() {
                confirm!(Template::pull(dependency, templates, dependencies, template_root));
            }
        }

        dependencies.insert(location.clone(), direct_dependencies);
        return success!(());
    }

    pub fn validate(&self, variant_registry: &VariantRegistry, templates: &Templates) -> Status<()> {
        for flavor in self.flavors.iter() {
            confirm!(flavor.validate(variant_registry, templates));
        }
        return success!(());
    }

    pub fn calculate_widthless(&mut self, templates: &Templates) -> bool {
        if self.widthless.is_some() {
            return false;
        }

        let mut guaranteed_width = true;
        for flavor in self.flavors.iter_mut() {
            if let Some(widthless) = flavor.calculate_widthless(templates) {
                if widthless {
                    self.widthless = Some(true);
                    return true;
                }
            } else {
                guaranteed_width = false;
            }
        }

        if guaranteed_width {
            self.widthless = Some(false);
            return true;
        }

        return false;
    }

    pub fn generate_start_list(&mut self, variant_registry: &VariantRegistry, templates: &Templates) {
        let mut token_list = Vec::new();
        let mut template_list = Vec::new();
        for flavor in self.flavors.iter() {
            flavor.generate_start_list(&mut token_list, &mut template_list, variant_registry, templates);
        }
        if !token_list.is_empty() {
            self.token_list = Some(token_list);
        }
        if !template_list.is_empty() {
            self.template_list = Some(template_list);
        }
    }

    pub fn create_widthless(&self, decisions: &mut SharedVector<Decision>, templates: &Templates) {
        for (flavor_index, flavor) in self.flavors.iter().enumerate() {
            if let Some(widthless) = flavor.widthless {
                if widthless {
                    decisions.push(Decision::Flavor(flavor_index));
                    flavor.create_widthless(decisions, templates);
                }
            }
        }
    }
}
