use rustler::types::tuple::get_tuple;
use rustler::NifStruct;
use rustler::{Binary, OwnedBinary};
use rustler::{Env, Term};

use erlang_term::RawTerm;
use erlang_term::Term as ElixirTerm;

use std::io::Write;

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
    match term_to_elixir_term(term) {
        Some(term) => Ok(bytes_to_binary(term.to_bytes(), env)),
        None => Err(Error::new(term.get_type())),
    }
}

fn term_to_elixir_term<'a>(term: Term<'a>) -> Option<ElixirTerm> {
    if let Ok(data) = term.atom_to_string() {
        return Some(ElixirTerm::Atom(data));
    }
    if let Ok(data) = term.decode::<u8>() {
        return Some(ElixirTerm::Byte(data));
    }
    if let Ok(data) = term.decode::<i32>() {
        return Some(ElixirTerm::Int(data));
    }
    if let Ok(data) = term.decode::<bool>() {
        return Some(ElixirTerm::Bool(data));
    }
    if let Ok(data) = term.decode::<f64>() {
        return Some(ElixirTerm::Float(data));
    }
    if term.is_number() {
        return Some(ElixirTerm::BigInt(
            format!("{:?}", term)
                .parse()
                .expect("number cannot be parsed to big int"),
        ));
    }
    if let Ok(data) = term.decode::<String>() {
        return Some(ElixirTerm::String(data));
    }
    if let Ok(data) = term.decode::<Binary>() {
        return Some(ElixirTerm::Bytes(data.as_slice().to_vec()));
    }
    if let Ok(data) = term.decode::<improper_list::ImproperListIterator>() {
        let list: Option<_> = data
            .map(|term| match term {
                improper_list::ListItem::Normal(term) => term_to_elixir_term(term),
                improper_list::ListItem::Improper(term) => Some(ElixirTerm::Other(
                    RawTerm::Improper(Box::new(term_to_elixir_term(term)?.into())),
                )),
            })
            .collect();
        return Some(ElixirTerm::List(list?));
    }
    if let Ok(data) = term.decode::<rustler::types::MapIterator>() {
        return Some(ElixirTerm::MapArbitrary(
            data.map(|(k, v)| {
                let key = term_to_elixir_term(k)?;
                let value = term_to_elixir_term(v)?;
                Some((key, value))
            })
            .collect::<Option<_>>()?,
        ));
    }
    if let Ok(data) = get_tuple(term) {
        return Some(ElixirTerm::Tuple(
            data.into_iter()
                .map(|x| term_to_elixir_term(x))
                .collect::<Option<_>>()?,
        ));
    }

    return None;
}

fn bytes_to_binary<'b>(data: Vec<u8>, env: Env<'b>) -> Term<'b> {
    // copied from str implementation
    let str_len = data.len();
    let mut bin = match OwnedBinary::new(str_len) {
        Some(bin) => bin,
        None => panic!("binary term allocation fail"),
    };
    bin.as_mut_slice()
        .write_all(&data)
        .expect("memory copy of string failed");
    bin.release(env).to_term(env)
}

rustler::init!("Elixir.EtfLoader", [to_binary]);
