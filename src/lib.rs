#[cfg(feature = "graph")]
pub mod graph;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::character::is_digit;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::IResult;
use ptable::Element;

#[derive(Debug, PartialEq, Eq, Clone)]
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
        tag(b"Na"),
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BranchedAtom {
    pub atom: Atom,
    pub ring_bonds: Vec<RingBond>,
    pub branches: Vec<Branch>,
}

fn branched_atom(input: &[u8]) -> IResult<&[u8], BranchedAtom> {
    map(
        tuple((atom, many0(ring_bond), many0(branch))),
        |(atom, ring_bonds, branches)| BranchedAtom {
            atom,
            ring_bonds,
            branches,
        },
    )(input)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Bond {
    Single,
    Double,
    Triple,
    Quadruple,
    Aromatic,
    Up,
    Down,
}

fn raw_bond(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(b"-"),
        tag(b"="),
        tag(b"#"),
        tag(b"$"),
        tag(b":"),
        tag(b"/"),
        tag(b"\\"),
    ))(input)
}

fn bond(input: &[u8]) -> IResult<&[u8], Bond> {
    map(raw_bond, |bnd: &[u8]| match bnd {
        b"-" => Bond::Single,
        b"=" => Bond::Double,
        b"#" => Bond::Triple,
        b"$" => Bond::Quadruple,
        b":" => Bond::Aromatic,
        b"/" => Bond::Up,
        b"\\" => Bond::Down,
        _ => unreachable!(),
    })(input)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RingBond {
    pub bond: Option<Bond>,
    pub ring_number: u8,
}

fn bond_digits(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(
        map_res(
            alt((
                take_while_m_n(1, 1, is_digit),
                preceded(tag(b"%"), take_while_m_n(2, 2, is_digit)),
            )),
            |s: &[u8]| std::str::from_utf8(s),
        ),
        |s: &str| s.parse::<u8>(),
    )(input)
}

fn ring_bond(input: &[u8]) -> IResult<&[u8], RingBond> {
    map(tuple((opt(bond), bond_digits)), |(bond, ring_number)| {
        RingBond { bond, ring_number }
    })(input)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Chain {
    pub chain: Option<Box<Chain>>,
    pub bond_or_dot: Option<BondOrDot>,
    pub branched_atom: BranchedAtom,
}

pub fn chain(input: &[u8]) -> IResult<&[u8], Chain> {
    map(
        tuple((branched_atom, opt(bond_or_dot), opt(chain))),
        |(branched_atom, bond_or_dot, chain)| Chain {
            chain: chain.map(|n| Box::new(n)),
            bond_or_dot,
            branched_atom,
        },
    )(input)
}

// Symbol for non-connected parts of compound
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dot;

fn dot(input: &[u8]) -> IResult<&[u8], Dot> {
    map(tag(b"."), |_| Dot)(input)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BondOrDot {
    Bond(Bond),
    Dot(Dot),
}

fn bond_or_dot(input: &[u8]) -> IResult<&[u8], BondOrDot> {
    alt((
        map(bond, |inner| BondOrDot::Bond(inner)),
        map(dot, |inner| BondOrDot::Dot(inner)),
    ))(input)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Branch {
    pub bond_or_dot: Option<BondOrDot>,
    pub chain: Chain,
}

fn branch(input: &[u8]) -> IResult<&[u8], Branch> {
    delimited(
        char('('),
        map(tuple((opt(bond_or_dot), chain)), |(bond_or_dot, chain)| {
            Branch { bond_or_dot, chain }
        }),
        char(')'),
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
    fn ring_bond_digit_cases() {
        assert_eq!(Ok(("".as_bytes(), 0u8)), bond_digits(b"0"));
        assert_eq!(Ok(("".as_bytes(), 12u8)), bond_digits(b"%12"));
        assert_eq!(Ok(("5".as_bytes(), 12u8)), bond_digits(b"%125"));
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
                        bond_or_dot: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                element: Element::Carbon
                            }),
                            ring_bonds: vec![],
                            branches: vec![]
                        }
                    })),
                    bond_or_dot: None,
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Carbon
                        }),
                        ring_bonds: vec![],
                        branches: vec![]
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
                        bond_or_dot: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                element: Element::Fluorine
                            }),
                            ring_bonds: vec![],
                            branches: vec![]
                        }
                    })),
                    bond_or_dot: None,
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Carbon
                        }),
                        ring_bonds: vec![],
                        branches: vec![]
                    }
                }
            )),
            chain(b"CF")
        );
    }

    #[test]
    fn chain_ethene() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Chain {
                    chain: Some(Box::new(Chain {
                        chain: None,
                        bond_or_dot: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                element: Element::Carbon
                            }),
                            ring_bonds: vec![],
                            branches: vec![]
                        }
                    })),
                    bond_or_dot: Some(BondOrDot::Bond(Bond::Double)),
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Carbon
                        }),
                        ring_bonds: vec![],
                        branches: vec![]
                    }
                }
            )),
            chain(b"C=C")
        );
    }

    // 1-Oxaspiro[2.5]octane
    #[test]
    fn ring_and_branch_chain() {
        let chain = chain(b"C1CCC2(CC1)CO2");
        assert!(chain.is_ok());
        assert!(chain.unwrap().0.is_empty());
    }

    // Isobutane
    #[test]
    fn branch_isobutane() {
        let chain = chain(b"CC(C)C");
        assert!(chain.is_ok());
        assert!(chain.unwrap().0.is_empty());
    }

    // Neopentane
    #[test]
    fn branch_neopentane() {
        let chain = chain(b"CC(C)(C)C");
        assert!(chain.is_ok());
        assert!(chain.unwrap().0.is_empty());
    }

    // Cyclopropyloxirane
    #[test]
    fn rings_chain() {
        let chain = chain(b"C1CC1C2CO2");
        println!("{:?}", chain);
        assert!(chain.is_ok());
        assert!(chain.unwrap().0.is_empty());
    }

    // TODO: proper structure for charge
    // #[test]
    // #[ignore]
    // fn chain_sodium_chloride() {
    // assert_eq!(
    // Ok((
    // "".as_bytes(),
    // Chain {
    // chain: Some(Box::new(Chain {
    // chain: None,
    // bond_or_dot: None,
    // branched_atom: BranchedAtom {
    // atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
    // element: Element::Carbon
    // }),
    // ring_bonds: vec![]
    // }
    // })),
    // bond_or_dot: Some(BondOrDot::Dot),
    // branched_atom: BranchedAtom {
    // atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
    // element: Element::Carbon
    // }),
    // ring_bonds: vec![]
    // }
    // }
    // )),
    // chain(b"[Na+].[Cl-]")
    // );
    // }
}
