use internal::*;
use debug::*;

use super::{ Decision, Piece, Templates };

#[derive(Debug, Clone)]
pub struct Flavor {
    pub pieces:     Vec<Piece>,
    pub widthless:  Option<bool>,
}

impl Flavor {

    pub fn new(pieces: Vec<Piece>) -> Self {
        Self {
            pieces:     pieces,
            widthless:  None,
        }
    }

    pub fn validate(&self, variant_registry: &VariantRegistry, templates: &Templates) -> Status<()> {
        for piece in self.pieces.iter() {
            confirm!(piece.validate(variant_registry, templates));
        }
        return success!(());
    }

    pub fn calculate_widthless(&mut self, templates: &Templates) -> Option<bool> {
        if self.widthless.is_some() {
            return self.widthless.clone();
        }

        let mut guaranteed_widthless = true;
        for piece in self.pieces.iter() {
            if let Some(widthless) = piece.calculate_widthless(templates) {
                if !widthless {
                    self.widthless = Some(false);
                    return Some(false);
                }
            } else {
                guaranteed_widthless = false;
            }
        }

        if guaranteed_widthless {
            self.widthless = Some(true);
            return Some(true);
        }

        return None;
    }

    pub fn generate_start_list(&self, token_list: &mut Vec<Data>, template_list: &mut Vec<Data>, variant_registry: &VariantRegistry, templates: &Templates) {
        for piece in self.pieces.iter() {
            if piece.generate_start_list(token_list, template_list, variant_registry, templates) {
                return;
            }
        }
    }

    pub fn create_widthless(&self, decisions: &mut Vector<Decision>, templates: &Templates) {
        for piece in self.pieces.iter() {
            piece.create_widthless(decisions, templates);
        }
    }
}
