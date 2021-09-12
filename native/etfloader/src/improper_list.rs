use rustler::Term;

pub enum ListItem<'a> {
    Normal(Term<'a>),
    Improper(Term<'a>),
}

impl<'a> ListItem<'a> {
    pub fn as_normal(self) -> Option<Term<'a>> {
        match self {
            ListItem::Normal(term) => Some(term),
            _ => None,
        }
    }
    pub fn as_improper(self) -> Option<Term<'a>> {
        match self {
            ListItem::Improper(term) => Some(term),
            _ => None,
        }
    }
}

pub struct ImproperListIterator<'a> {
    term: Term<'a>,
    finished: bool,
}

impl<'a> ImproperListIterator<'a> {
    pub fn new(term: Term<'a>) -> Option<Self> {
        if term.is_list() || term.is_empty_list() {
            let iter = ImproperListIterator {
                term,
                finished: false,
            };
            Some(iter)
        } else {
            None
        }
    }
}

impl<'a> Iterator for ImproperListIterator<'a> {
    type Item = ListItem<'a>;

    fn next(&mut self) -> Option<ListItem<'a>> {
        if self.finished {
            return None;
        }
        let cell = self.term.list_get_cell();

        match cell {
            Ok((head, tail)) => {
                self.term = tail;
                Some(ListItem::Normal(head))
            }
            Err(_) => {
                self.finished = true;
                if self.term.is_empty_list() {
                    None
                } else {
                    Some(ListItem::Improper(self.term))
                }
            }
        }
    }
}

impl<'a> rustler::Decoder<'a> for ImproperListIterator<'a> {
    fn decode(term: Term<'a>) -> rustler::NifResult<Self> {
        match ImproperListIterator::new(term) {
            Some(iter) => Ok(iter),
            None => Err(rustler::Error::BadArg),
        }
    }
}
