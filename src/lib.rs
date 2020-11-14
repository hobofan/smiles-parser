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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum Symbol {
    ElementSymbol(Element),
    // AromaticSymbol not supported
    Unknown,
}

fn raw_symbol(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(b"*"),
        // Two letter symbols have to appear before one letter symbols or they won't be recognized
        alt((
            tag(b"Ac"),
            tag(b"Ag"),
            tag(b"Al"),
            tag(b"Am"),
            tag(b"Ar"),
            tag(b"As"),
            tag(b"At"),
            tag(b"Au"),
            tag(b"Ba"),
            tag(b"Be"),
            tag(b"Bh"),
            tag(b"Bi"),
            tag(b"Bk"),
            tag(b"Br"),
            tag(b"Ca"),
            tag(b"Cd"),
            tag(b"Ce"),
            tag(b"Cf"),
            tag(b"Cl"),
            tag(b"Cm"),
        )),
        alt((
            tag(b"Cn"),
            tag(b"Co"),
            tag(b"Cr"),
            tag(b"Cs"),
            tag(b"Cu"),
            tag(b"Db"),
            tag(b"Ds"),
            tag(b"Dy"),
            tag(b"Er"),
            tag(b"Es"),
            tag(b"Eu"),
            tag(b"Fe"),
            tag(b"Fl"),
            tag(b"Fm"),
            tag(b"Fr"),
            tag(b"Ga"),
            tag(b"Gd"),
            tag(b"Ge"),
            tag(b"He"),
            tag(b"Hf"),
        )),
        alt((
            tag(b"Hg"),
            tag(b"Ho"),
            tag(b"Hs"),
            tag(b"In"),
            tag(b"Ir"),
            tag(b"Kr"),
            tag(b"La"),
            tag(b"Li"),
            tag(b"Lr"),
            tag(b"Lu"),
            tag(b"Lv"),
            tag(b"Mc"),
            tag(b"Md"),
            tag(b"Mg"),
            tag(b"Mn"),
            tag(b"Mo"),
            tag(b"Mt"),
            tag(b"Na"),
            tag(b"Nb"),
            tag(b"Nd"),
        )),
        alt((
            tag(b"Ne"),
            tag(b"Nh"),
            tag(b"Ni"),
            tag(b"No"),
            tag(b"Np"),
            tag(b"Og"),
            tag(b"Os"),
            tag(b"Pa"),
            tag(b"Pb"),
            tag(b"Pd"),
            tag(b"Pm"),
            tag(b"Po"),
            tag(b"Pr"),
            tag(b"Pt"),
            tag(b"Pu"),
            tag(b"Ra"),
            tag(b"Rb"),
            tag(b"Re"),
            tag(b"Rf"),
            tag(b"Rg"),
        )),
        alt((
            tag(b"Rh"),
            tag(b"Rn"),
            tag(b"Ru"),
            tag(b"Sb"),
            tag(b"Sc"),
            tag(b"Se"),
            tag(b"Sg"),
            tag(b"Si"),
            tag(b"Sm"),
            tag(b"Sn"),
            tag(b"Sr"),
            tag(b"Ta"),
            tag(b"Tb"),
            tag(b"Tc"),
            tag(b"Te"),
            tag(b"Th"),
            tag(b"Ti"),
            tag(b"Tl"),
            tag(b"Tm"),
            tag(b"Ts"),
        )),
        alt((tag(b"Xe"), tag(b"Yb"), tag(b"Zn"), tag(b"Zr"))),
        // Single letter
        alt((
            tag(b"B"),
            tag(b"C"),
            tag(b"F"),
            tag(b"H"),
            tag(b"I"),
            tag(b"K"),
            tag(b"N"),
            tag(b"O"),
            tag(b"P"),
            tag(b"S"),
            tag(b"U"),
            tag(b"V"),
            tag(b"W"),
            tag(b"Y"),
        )),
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct BracketAtom {
    pub isotope: Option<u16>,
    pub symbol: Symbol,
    // TODO: chiral?
    pub hcount: u8,
    pub charge: i8,
    // TODO: class?
}

fn charge(input: &[u8]) -> IResult<&[u8], i8> {
    map(
        many0(map(
            tuple((
                alt((tag("+"), tag("-"))),
                opt(map_res(
                    map_res(take_while_m_n(1, 2, is_digit), |s: &[u8]| {
                        std::str::from_utf8(s)
                    }),
                    |s: &str| s.parse::<u8>(),
                )),
            )),
            |(tag, count): (&[u8], Option<u8>)| {
                let count = count.unwrap_or(1) as i8;
                if tag[0] == b'+' {
                    count
                } else {
                    -count
                }
            },
        )),
        |v| v.into_iter().fold(0, |acc, x| acc + x),
    )(input)
}

fn hcount(input: &[u8]) -> IResult<&[u8], u8> {
    map(
        opt(map(
            tuple((
                tag("H"),
                opt(map_res(
                    map_res(take_while_m_n(1, 1, is_digit), |s: &[u8]| {
                        std::str::from_utf8(s)
                    }),
                    |s: &str| s.parse::<u8>(),
                )),
            )),
            |(_, count): (&[u8], Option<u8>)| count.unwrap_or(1),
        )),
        |res| res.unwrap_or(0),
    )(input)
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
            tuple((isotope_opt, symbol, hcount, charge)),
            |(isotope, sym, hcount, charge): (Option<u16>, Symbol, u8, i8)| BracketAtom {
                isotope,
                symbol: sym,
                hcount,
                charge,
            },
        ),
        char(']'),
    )(input)
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Hash)]
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
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub struct Dot;

fn dot(input: &[u8]) -> IResult<&[u8], Dot> {
    map(tag(b"."), |_| Dot)(input)
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Hash)]
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
                    hcount: 0,
                    charge: -2,
                }
            )),
            bracket_atom(b"[16C--]")
        );
        assert_eq!(
            Ok((
                "CC".as_bytes(),
                BracketAtom {
                    isotope: Some(16),
                    symbol: Symbol::ElementSymbol(Element::Carbon),
                    hcount: 1,
                    charge: 3,
                }
            )),
            bracket_atom(b"[16CH+3]CC")
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
                    hcount: 0,
                    charge: 0,
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
