use rustler::NifStruct;
use rustler::{Atom, Binary};
use rustler::{Env, Term};

use erlang_term::RawTerm;

pub mod atoms;
pub mod convert;
pub mod improper_list;

#[derive(Debug, NifStruct)]
#[module = "EtfLoader.Error"]
struct Error {
    __exception__: bool,
    message: String,
    r#type: Option<String>,
    value: (),
}

impl Error {
    pub fn new(typing: rustler::TermType) -> Error {
        Error {
            __exception__: true,
            message: format!("cannot format term of type: '{:?}'", typing),
            r#type: Some(format!("{:?}", typing)),
            value: (),
        }
    }
}

#[rustler::nif]
fn to_binary<'a>(env: Env<'a>, term: Term<'a>) -> Result<Term<'a>, Error> {
    match convert::term_to_elixir_term(term) {
        Some(term) => Ok(convert::bytes_to_binary(term.to_bytes(), env)),
        None => Err(Error::new(term.get_type())),
    }
}

#[rustler::nif(name = "internal_from_binary")]
fn from_binary<'a>(
    env: Env<'a>,
    binary: Binary,
    opts: convert::FromBinaryOptions,
) -> Result<Term<'a>, Atom> {
    match RawTerm::from_bytes(binary.as_slice()) {
        Ok(term) => match convert::raw_term_to_term(term, env, &opts) {
            Some(term) => Ok(term),
            None => Err(atoms::binary_cannot_be_loaded()),
        },
        Err(_) => Err(atoms::invalid_binary()),
    }
}

rustler::init!("Elixir.EtfLoader", [to_binary, from_binary]);
