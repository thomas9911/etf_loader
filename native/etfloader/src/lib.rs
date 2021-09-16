use rustler::types::atom::nil;
use rustler::types::tuple::{get_tuple, make_tuple};
use rustler::{Atom, Binary, OwnedBinary};
use rustler::{Decoder, Encoder, Env, Term};
use rustler::{NifResult, NifStruct};

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

rustler::atoms! {
    binary_cannot_be_loaded,
    invalid_binary,
    unsafe_atom
}

#[derive(Debug)]
struct FromBinaryOptions {
    unsafe_atom: bool,
}

impl<'a> From<Vec<(Atom, Term<'a>)>> for FromBinaryOptions {
    fn from(input: Vec<(Atom, Term<'a>)>) -> FromBinaryOptions {
        let unsafe_atom = input
            .iter()
            .find_map(|&(x, y)| {
                if x == unsafe_atom() {
                    match y.decode::<bool>() {
                        Ok(x) => Some(x),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .unwrap_or(false);

        FromBinaryOptions { unsafe_atom }
    }
}

impl<'a> Decoder<'a> for FromBinaryOptions {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        match term.decode::<Vec<(Atom, Term<'a>)>>() {
            Ok(iter) => Ok(FromBinaryOptions::from(iter)),
            Err(_) => Err(rustler::Error::BadArg),
        }
    }
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

#[rustler::nif]
fn from_binary<'a>(
    env: Env<'a>,
    binary: Binary,
    opts: FromBinaryOptions,
) -> Result<Term<'a>, Atom> {
    match RawTerm::from_bytes(binary.as_slice()) {
        Ok(term) => match raw_term_to_term(term, env, &opts) {
            Some(term) => Ok(term),
            None => Err(binary_cannot_be_loaded()),
        },
        Err(_) => Err(invalid_binary()),
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

fn raw_term_to_term<'a>(term: RawTerm, env: Env<'a>, opts: &FromBinaryOptions) -> Option<Term<'a>> {
    use RawTerm::*;
    match term {
        SmallInt(x) => Some(x.encode(env)),
        Int(x) => Some(x.encode(env)),
        Nil => Some(Vec::<()>::new().encode(env)),
        String(x) => Some(x.encode(env)),
        Float(x) => Some(x.encode(env)),
        Atom(x) | SmallAtom(x) | AtomDeprecated(x) | SmallAtomDeprecated(x) if x == "false" => {
            Some(false.encode(env))
        }
        Atom(x) | SmallAtom(x) | AtomDeprecated(x) | SmallAtomDeprecated(x) if x == "true" => {
            Some(true.encode(env))
        }
        Atom(x) | SmallAtom(x) | AtomDeprecated(x) | SmallAtomDeprecated(x) if x == "nil" => {
            Some(nil().encode(env))
        }
        Atom(x) | SmallAtom(x) | AtomDeprecated(x) | SmallAtomDeprecated(x) if opts.unsafe_atom => {
            match rustler::Atom::from_bytes(env, x.as_bytes()) {
                Ok(atom) => Some(atom.to_term(env)),
                _ => None,
            }
        }
        Atom(x) | SmallAtom(x) | AtomDeprecated(x) | SmallAtomDeprecated(x) => {
            match rustler::Atom::try_from_bytes(env, x.as_bytes()) {
                Ok(Some(atom)) => Some(atom.to_term(env)),
                Ok(None) => Some(x.encode(env)),
                _ => None,
            }
        }
        Binary(x) => Some(bytes_to_binary(x, env)),
        List(list) => {
            let mut improper = None;
            let data =
                list.into_iter()
                    .try_fold(Vec::new(), |mut acc, x| -> Option<Vec<Term<'a>>> {
                        match x {
                            Improper(y) => {
                                improper = Some(raw_term_to_term(*y, env, opts)?);
                                Some(acc)
                            }
                            _ => {
                                acc.push(raw_term_to_term(x, env, opts)?);
                                Some(acc)
                            }
                        }
                    })?;
            if let Some(tail) = improper {
                let term = data.into_iter().rfold(tail, |acc, x| acc.list_prepend(x));
                Some(term)
            } else {
                Some(data.encode(env))
            }
        }
        SmallTuple(list) | LargeTuple(list) => {
            let data: Option<Vec<_>> = list
                .into_iter()
                .map(|x| raw_term_to_term(x, env, opts))
                .collect();
            Some(make_tuple(env, &data?))
        }
        Improper(x) => raw_term_to_term(*x, env, opts),
        Map(map) => {
            let (keys, values): (Vec<_>, Vec<_>) = map.into_iter().unzip();
            let keys: Option<Vec<_>> = keys
                .into_iter()
                .map(|x| raw_term_to_term(x, env, opts))
                .collect();
            let values: Option<Vec<_>> = values
                .into_iter()
                .map(|x| raw_term_to_term(x, env, opts))
                .collect();
            Term::map_from_arrays(env, &keys?, &values?).ok()
        }
        // SmallBigInt(BigInt),
        // LargeBigInt(BigInt),
        // Pid {
        //     node: Box<RawTerm>,
        //     id: u32,
        //     serial: u32,
        //     creation: u8,
        // },
        // Port {
        //     node: Box<RawTerm>,
        //     id: u32,
        //     creation: u8,
        // },
        // Ref {
        //     node: Box<RawTerm>,
        //     id: Vec<u32>,
        //     creation: u8,
        // },
        // Function {
        //     size: u32,
        //     arity: u8,
        //     uniq: [u8; 16],
        //     index: u32,
        //     module: Box<RawTerm>,
        //     old_index: Box<RawTerm>,
        //     old_uniq: Box<RawTerm>,
        //     pid: Box<RawTerm>,
        //     free_var: Vec<RawTerm>,
        // },
        _ => None,
    }
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

rustler::init!("Elixir.EtfLoader", [to_binary, from_binary]);
