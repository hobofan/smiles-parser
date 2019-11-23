#[macro_use]
extern crate nom;

use basechem::Element;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::is_digit;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub enum Symbol {
    ElementSymbol(Element),
    // AromaticSymbol not supported
    Unknown,
}

fn raw_symbol(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(b"*"),
        // Two letter symbols have to appear before one letter symbols or they won't be recognized
        tag(b"He"),
        tag(b"H"),
        tag(b"C"),
        // TODO: full list
    ))(input)
}

fn symbol(input: &[u8]) -> IResult<&[u8], Symbol> {
    map_res(raw_symbol, |sym: &[u8]| match sym {
        b"*" => Ok(Symbol::Unknown),
        other => {
            let other_str = std::str::from_utf8(other).map_err(|_| "Unparsable UTF-8")?;
            let try_element = Element::from_symbol(other_str);
            try_element
                .ok_or("Unknown element symbol")
                .map(|element| Symbol::ElementSymbol(element))
        }
    })(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct BracketAtom {
    pub isotope: Option<u16>,
    pub symbol: Symbol,
    // TODO: chiral?
    // TODO: hcount?
    // TODO: charge?
    // TODO: class?
}

fn isotope_opt(input: &[u8]) -> IResult<&[u8], Option<u16>> {
    opt(map_res(
        map_res(take_while_m_n(1, 3, is_digit), |s: &[u8]| {
            std::str::from_utf8(s)
        }),
        |s: &str| s.parse::<u16>(),
    ))(input)
}

fn bracket_atom(input: &[u8]) -> IResult<&[u8], BracketAtom> {
    delimited(
        char('['),
        map(
            tuple((isotope_opt, symbol)),
            |(isotope, sym): (Option<u16>, Symbol)| BracketAtom {
                isotope,
                symbol: sym,
            },
        ),
        char(']'),
    )(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct AliphaticOrganicAtom {
    pub element: Element,
}

fn raw_aliphatic_organic(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        // Two letter symbols have to appear before one letter symbols or they won't be recognized
        tag(b"Cl"),
        tag(b"Br"),
        tag(b"B"),
        tag(b"C"),
        tag(b"N"),
        tag(b"O"),
        tag(b"S"),
        tag(b"P"),
        tag(b"F"),
        tag(b"I"),
    ))(input)
}

fn aliphatic_organic_atom(input: &[u8]) -> IResult<&[u8], AliphaticOrganicAtom> {
    map_res(raw_aliphatic_organic, |sym: &[u8]| {
        let other_str = std::str::from_utf8(sym).map_err(|_| "Unparsable UTF-8")?;
        let try_element = Element::from_symbol(other_str);
        try_element
            .ok_or("Unknown element symbol")
            .map(|element| AliphaticOrganicAtom { element })
    })(input)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Atom {
    Bracket(BracketAtom),
    AliphaticOrganic(AliphaticOrganicAtom),
    // AromaticOrganic not supported
    Unknown,
}

fn atom(input: &[u8]) -> IResult<&[u8], Atom> {
    alt((
        map(tag(b"*"), |_| Atom::Unknown),
        map(bracket_atom, |inner| Atom::Bracket(inner)),
        map(aliphatic_organic_atom, |inner| {
            Atom::AliphaticOrganic(inner)
        }),
    ))(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct BranchedAtom {
    atom: Atom,
    // TODO: ring_bonds: Vec<RingBond>,
    // TODO: branches: Vec<Branch>,
}

fn branched_atom(input: &[u8]) -> IResult<&[u8], BranchedAtom> {
    map(atom, |atom| BranchedAtom { atom })(input)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Chain {
    chain: Option<Box<Chain>>,
    branched_atom: BranchedAtom,
    // TODO: other fields
}

fn chain(input: &[u8]) -> IResult<&[u8], Chain> {
    map(
        tuple((branched_atom, opt(chain))),
        |(branched_atom, chain)| Chain {
            chain: chain.map(|n| Box::new(n)),
            branched_atom,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_cases() {
        assert_eq!(Ok(("".as_bytes(), Symbol::Unknown)), symbol(b"*"));
        assert_eq!(
            Ok(("".as_bytes(), Symbol::ElementSymbol(Element::Helium))),
            symbol(b"He")
        );
    }

    #[test]
    fn isotope_opt_cases() {
        assert_eq!(Ok(("".as_bytes(), Some(0u16))), isotope_opt(b"0"));
        assert_eq!(Ok(("".as_bytes(), Some(125u16))), isotope_opt(b"125"));
        assert_eq!(Ok(("X".as_bytes(), Some(125u16))), isotope_opt(b"125X"));
        assert_eq!(Ok(("7".as_bytes(), Some(125u16))), isotope_opt(b"1257"));
    }

    #[test]
    fn bracket_atom_cases() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                BracketAtom {
                    isotope: Some(16),
                    symbol: Symbol::ElementSymbol(Element::Carbon),
                }
            )),
            bracket_atom(b"[16C]")
        );
        assert_eq!(
            Ok((
                "CC".as_bytes(),
                BracketAtom {
                    isotope: Some(16),
                    symbol: Symbol::ElementSymbol(Element::Carbon),
                }
            )),
            bracket_atom(b"[16C]CC")
        );
    }

    #[test]
    fn atom_cases() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Atom::Bracket(BracketAtom {
                    isotope: Some(16),
                    symbol: Symbol::ElementSymbol(Element::Carbon),
                })
            )),
            atom(b"[16C]")
        );
        assert_eq!(Ok(("".as_bytes(), Atom::Unknown)), atom(b"*"));
    }

    #[test]
    fn chain_ethane() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Chain {
                    chain: Some(Box::new(Chain {
                        chain: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                element: Element::Carbon
                            })
                        }
                    })),
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Carbon
                        })
                    }
                }
            )),
            chain(b"CC")
        );
    }

    #[test]
    fn chain_fluoromethane() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Chain {
                    chain: Some(Box::new(Chain {
                        chain: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                element: Element::Fluorine
                            })
                        }
                    })),
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Carbon
                        })
                    }
                }
            )),
            chain(b"CF")
        );
    }
}
