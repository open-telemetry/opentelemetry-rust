use crate::api;

pub trait Provider {
    type Tracer: api::Tracer;
    fn get_tracer<S: Into<String>>(&self, name: S) -> Self::Tracer;
}
