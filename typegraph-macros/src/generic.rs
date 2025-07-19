use std::collections::HashMap;

use syn::visit_mut::{self, VisitMut};
use syn::{Ident, Type};

pub struct Substitution(HashMap<Ident, Type>);

impl Substitution {
    pub fn new(v: Vec<Type>, g: &syn::Generics) -> Self {
        Self(g.type_params().map(|tp| tp.ident.clone()).zip(v).collect())
    }

    pub fn substitute_all(&mut self, mut fn_types: Vec<Type>) -> Vec<Type> {
        for ty in fn_types.iter_mut() {
            self.visit_type_mut(ty);
        }

        fn_types
    }

    pub fn substitute(&mut self, mut ty: Type) -> Type {
        self.visit_type_mut(&mut ty);
        ty
    }
}

impl VisitMut for Substitution {
    fn visit_type_path_mut(&mut self, i: &mut syn::TypePath) {
        if let Some(segment) = i.path.segments.last_mut() {
            if let Some(Type::Path(ref_path)) = self.0.get(&segment.ident) {
                *i = ref_path.clone();
            }
        }

        visit_mut::visit_type_path_mut(self, i);
    }
}
