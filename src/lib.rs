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
    AromaticSymbol(Element),
    Unknown,
}

fn raw_symbol(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        // Unknown
        tag(b"*"),
        // ElementSymbol
        alt((
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
        )),
        // AromaticSymbol
        alt((
            tag(b"se"),
            tag(b"as"),
            //
            tag(b"b"),
            tag(b"c"),
            tag(b"n"),
            tag(b"o"),
            tag(b"p"),
            tag(b"s"),
        )),
    ))(input)
}

fn symbol(input: &[u8]) -> IResult<&[u8], Symbol> {
    map_res(raw_symbol, |sym: &[u8]| match sym {
        b"*" => Ok(Symbol::Unknown),
        b"se" | b"as" | b"b" | b"c" | b"n" | b"o" | b"p" | b"s" => Ok(match sym {
            b"se" => Symbol::AromaticSymbol(Element::Selenium),
            b"as" => Symbol::AromaticSymbol(Element::Arsenic),
            b"b" => Symbol::AromaticSymbol(Element::Boron),
            b"c" => Symbol::AromaticSymbol(Element::Carbon),
            b"n" => Symbol::AromaticSymbol(Element::Nitrogen),
            b"o" => Symbol::AromaticSymbol(Element::Oxygen),
            b"p" => Symbol::AromaticSymbol(Element::Phosphorus),
            b"s" => Symbol::AromaticSymbol(Element::Sulfur),
            _ => unreachable!(),
        }),
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
    pub chiral: Option<Chirality>,
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
            tuple((isotope_opt, symbol, opt(chirality), hcount, charge)),
            |(isotope, sym, chiral, hcount, charge): (
                Option<u16>,
                Symbol,
                Option<Chirality>,
                u8,
                i8,
            )| BracketAtom {
                isotope,
                symbol: sym,
                chiral,
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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum Chirality {
    /// `@`
    Anticlockwise,
    /// `@@`
    Clockwise,
    /// `@TH1`, `@TH2`
    Tetrahedral(u8),
    /// `@AL1`, `@AL2`
    Allenal(u8),
    /// `@SP1`, `@SP2`, `@SP3`
    SquarePlanar(u8),
    /// `@TB1` ... `@TB20`
    TrigonalBipyramidal(u8),
    /// `@OH1` ... `@OH30`
    Octahedral(u8),
}

fn raw_chirality(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        alt((tag(b"@TH1"), tag(b"@TH2"))),
        alt((tag(b"@AL1"), tag(b"@AL2"))),
        alt((tag(b"@SP1"), tag(b"@SP2"), tag(b"@SP3"))),
        alt((
            tag(b"@TB10"),
            tag(b"@TB11"),
            tag(b"@TB12"),
            tag(b"@TB13"),
            tag(b"@TB14"),
            tag(b"@TB15"),
            tag(b"@TB16"),
            tag(b"@TB17"),
            tag(b"@TB18"),
            tag(b"@TB19"),
            tag(b"@TB20"),
            tag(b"@TB1"),
            tag(b"@TB2"),
            tag(b"@TB3"),
            tag(b"@TB4"),
            tag(b"@TB5"),
            tag(b"@TB6"),
            tag(b"@TB7"),
            tag(b"@TB8"),
            tag(b"@TB9"),
        )),
        alt((
            tag(b"@OH10"),
            tag(b"@OH11"),
            tag(b"@OH12"),
            tag(b"@OH13"),
            tag(b"@OH14"),
            tag(b"@OH15"),
            tag(b"@OH16"),
            tag(b"@OH17"),
            tag(b"@OH18"),
            tag(b"@OH19"),
            tag(b"@OH20"),
            tag(b"@OH1"),
            tag(b"@OH2"),
            tag(b"@OH3"),
            tag(b"@OH4"),
            tag(b"@OH5"),
            tag(b"@OH6"),
            tag(b"@OH7"),
            tag(b"@OH8"),
            tag(b"@OH9"),
        )),
        alt((
            tag(b"@OH21"),
            tag(b"@OH22"),
            tag(b"@OH23"),
            tag(b"@OH24"),
            tag(b"@OH25"),
            tag(b"@OH26"),
            tag(b"@OH27"),
            tag(b"@OH28"),
            tag(b"@OH29"),
            tag(b"@OH30"),
        )),
        tag(b"@@"),
        tag(b"@"),
    ))(input)
}

fn chirality(input: &[u8]) -> IResult<&[u8], Chirality> {
    map_res(raw_chirality, |sym: &[u8]| {
        let other_str = std::str::from_utf8(sym).map_err(|_| "Unparsable UTF-8")?;

        let chirality: Result<Chirality, &'static str> = match other_str {
            "@" => Ok(Chirality::Anticlockwise),
            "@@" => Ok(Chirality::Clockwise),
            "@TH1" | "@TH2" => Ok(Chirality::Tetrahedral(other_str[3..].parse().unwrap())),
            "@AL1" | "@AL2" => Ok(Chirality::Allenal(other_str[3..].parse().unwrap())),
            "@SP1" | "@SP2" | "@SP3" => {
                Ok(Chirality::SquarePlanar(other_str[3..].parse().unwrap()))
            }
            "@TB1" | "@TB2" | "@TB3" | "@TB4" | "@TB5" | "@TB6" | "@TB7" | "@TB8" | "@TB9"
            | "@TB10" | "@TB11" | "@TB12" | "@TB13" | "@TB14" | "@TB15" | "@TB16" | "@TB17"
            | "@TB18" | "@TB19" | "@TB20" => Ok(Chirality::TrigonalBipyramidal(
                other_str[3..].parse().unwrap(),
            )),
            "@OH1" | "@OH2" | "@OH3" | "@OH4" | "@OH5" | "@OH6" | "@OH7" | "@OH8" | "@OH9"
            | "@OH10" | "@OH11" | "@OH12" | "@OH13" | "@OH14" | "@OH15" | "@OH16" | "@OH17"
            | "@OH18" | "@OH19" | "@OH20" | "@OH21" | "@OH22" | "@OH23" | "@OH24" | "@OH25"
            | "@OH26" | "@OH27" | "@OH28" | "@OH29" | "@OH30" => {
                Ok(Chirality::Octahedral(other_str[3..].parse().unwrap()))
            }
            _ => unreachable!(),
        };

        chirality
    })(input)
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
                    chiral: None,
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
                    chiral: None,
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
    fn chirality_cases() {
        assert_eq!(
            Ok(("".as_bytes(), Chirality::Anticlockwise)),
            chirality(b"@")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::Tetrahedral(1))),
            chirality(b"@TH1")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::Allenal(2))),
            chirality(b"@AL2")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::SquarePlanar(3))),
            chirality(b"@SP3")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::TrigonalBipyramidal(1))),
            chirality(b"@TB1")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::TrigonalBipyramidal(11))),
            chirality(b"@TB11")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::Octahedral(1))),
            chirality(b"@OH1")
        );
        assert_eq!(
            Ok(("".as_bytes(), Chirality::Octahedral(11))),
            chirality(b"@OH11")
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
                    chiral: None,
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

    #[test]
    fn chain_trigonal_bipyramidal() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Chain {
                    chain: Some(Box::new(Chain {
                        chain: Some(Box::new(Chain {
                            chain: None,
                            bond_or_dot: None,
                            branched_atom: BranchedAtom {
                                atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                    element: Element::Nitrogen
                                }),
                                ring_bonds: vec![],
                                branches: vec![]
                            }
                        })),
                        bond_or_dot: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::Bracket(BracketAtom {
                                isotope: None,
                                symbol: Symbol::ElementSymbol(Element::Arsenic),
                                chiral: Some(Chirality::TrigonalBipyramidal(15)),
                                hcount: 0,
                                charge: 0,
                            }),
                            ring_bonds: vec![],
                            branches: vec![
                                Branch {
                                    bond_or_dot: None,
                                    chain: Chain {
                                        chain: None,
                                        bond_or_dot: None,
                                        branched_atom: BranchedAtom {
                                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                                element: Element::Chlorine
                                            }),
                                            ring_bonds: vec![],
                                            branches: vec![]
                                        }
                                    },
                                },
                                Branch {
                                    bond_or_dot: None,
                                    chain: Chain {
                                        chain: None,
                                        bond_or_dot: None,
                                        branched_atom: BranchedAtom {
                                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                                element: Element::Sulfur
                                            }),
                                            ring_bonds: vec![],
                                            branches: vec![]
                                        }
                                    },
                                },
                                Branch {
                                    bond_or_dot: None,
                                    chain: Chain {
                                        chain: None,
                                        bond_or_dot: None,
                                        branched_atom: BranchedAtom {
                                            atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                                                element: Element::Bromine
                                            }),
                                            ring_bonds: vec![],
                                            branches: vec![]
                                        }
                                    },
                                },
                            ]
                        }
                    })),
                    bond_or_dot: None,
                    branched_atom: BranchedAtom {
                        atom: Atom::AliphaticOrganic(AliphaticOrganicAtom {
                            element: Element::Fluorine
                        }),
                        ring_bonds: vec![],
                        branches: vec![]
                    }
                }
            )),
            chain(b"F[As@TB15](Cl)(S)(Br)N")
        );
    }

    #[test]
    fn chain_sodium_chloride() {
        assert_eq!(
            Ok((
                "".as_bytes(),
                Chain {
                    chain: Some(Box::new(Chain {
                        chain: None,
                        bond_or_dot: None,
                        branched_atom: BranchedAtom {
                            atom: Atom::Bracket(BracketAtom {
                                isotope: None,
                                symbol: Symbol::ElementSymbol(Element::Chlorine),
                                chiral: None,
                                hcount: 0,
                                charge: -1,
                            }),
                            ring_bonds: vec![],
                            branches: vec![]
                        }
                    })),
                    bond_or_dot: Some(BondOrDot::Dot(Dot)),
                    branched_atom: BranchedAtom {
                        atom: Atom::Bracket(BracketAtom {
                            isotope: None,
                            symbol: Symbol::ElementSymbol(Element::Sodium),
                            chiral: None,
                            hcount: 0,
                            charge: 1,
                        }),
                        ring_bonds: vec![],
                        branches: vec![]
                    }
                }
            )),
            chain(b"[Na+].[Cl-]")
        );
    }
}
