use crisp::{
    env::Environment,
    types::{Symbol, Value},
};

use crate::{Slide, SlideBuilder};

pub fn setup_env() {
    let mut env = Environment::new();
    env.insert_symbol(
        Symbol::from("slide"),
        Value::func(|args| Ok(Value::from(args))),
    )
}
