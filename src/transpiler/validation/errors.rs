// src/validation/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParameterFlowError {
    #[error("Parameter '{parameter}' is consumed but never produced in protocol '{protocol}'")]
    UnproducedParameter { parameter: String, protocol: String },

    #[error(
        "Parameter '{parameter}' has multiple producers {producers:?} in protocol '{protocol}'"
    )]
    MultipleProducers {
        parameter: String,
        producers: Vec<String>,
        protocol: String,
    },

    #[error("Circular dependency detected in protocol '{protocol}': {cycle}")]
    CircularDependency { protocol: String, cycle: String },

    #[error("Parameter '{parameter}' used in interaction '{interaction}' is not declared in protocol '{protocol}'")]
    UndeclaredParameter {
        parameter: String,
        interaction: String,
        protocol: String,
    },
}
