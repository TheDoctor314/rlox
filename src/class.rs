use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct LoxClass {
    name: String,
}

impl LoxClass {
    pub(crate) fn new(name: String) -> Self {
        Self { name }
    }
}
impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxInstance {
    class: Rc<LoxClass>,
}

impl LoxInstance {
    pub(crate) fn new(class: &Rc<LoxClass>) -> Self {
        Self {
            class: Rc::clone(class),
        }
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
