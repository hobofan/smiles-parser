use derive_more::{AsRef, Deref, DerefMut};
use itertools::Itertools;
use petgraph::algo::astar;
use petgraph::graph::NodeIndex;
use petgraph::visit::{IntoNodeIdentifiers, NodeFiltered};
use petgraph::{Graph, Undirected};
use ptable::Element;

use crate::{AliphaticOrganicAtom, Bond, BondOrDot, Chain};

#[derive(Debug, Clone)]
pub enum Atom {
    AliphaticOrganic(AliphaticOrganicAtom),
    Element(Element),
}

impl Into<Atom> for crate::Atom {
    fn into(self) -> Atom {
        match self {
            crate::Atom::AliphaticOrganic(inner) => Atom::AliphaticOrganic(inner),
            _ => panic!(),
        }
    }
}

#[derive(Default, Clone, AsRef, Deref, DerefMut)]
pub struct MoleculeGraph(Graph<Atom, Bond, Undirected>);

impl MoleculeGraph {
    pub fn from_chain(chain: Chain) -> Self {
        let mut graph = MoleculeGraph::default();

        fn add_chain_to_graph(
            graph: &mut MoleculeGraph,
            chain: &Chain,
            previous_node: Option<NodeIndex>,
            branch_bond: Option<Bond>,
        ) {
            let branched_atom = chain.branched_atom.clone();
            let current_node = graph.add_node(branched_atom.atom.into());
            if let Some(previous_node) = previous_node {
                let mut bond = branch_bond;
                if bond.is_none() {
                    bond = Some(
                        chain
                            .bond_or_dot
                            .as_ref()
                            .map(|n| match n {
                                BondOrDot::Bond(bond) => Some(bond),
                                _ => None,
                            })
                            .flatten()
                            .unwrap_or(&Bond::Single)
                            .to_owned(),
                    );
                }
                let bond = bond.unwrap();
                graph.add_edge(current_node, previous_node, bond.clone());
            }

            if let Some(chain) = &chain.chain {
                add_chain_to_graph(graph, &*chain, Some(current_node), None);
            }

            for branch in branched_atom.branches {
                let branch_bond = branch
                    .bond_or_dot
                    .clone()
                    .map(|n| match n {
                        BondOrDot::Bond(bond) => Some(bond),
                        _ => None,
                    })
                    .flatten();
                add_chain_to_graph(graph, &branch.chain, Some(current_node), branch_bond)
            }
        }

        fn fill_graph_with_hydrogen(graph: &mut MoleculeGraph) {
            for atom_index in graph.node_indices() {
                let atom = graph.node_weight(atom_index).unwrap();

                let desired_bonds_num = match atom {
                    Atom::AliphaticOrganic(atom) => match atom.element {
                        Element::Carbon => Some(4),
                        Element::Phosphorus => Some(5),
                        Element::Oxygen => Some(2),
                        _ => None,
                    },
                    _ => None,
                }
                .expect("Can't handle this atom yet");

                let neighbor_edges = graph.edges(atom_index).collect::<Vec<_>>();
                let current_bonds_num: usize = neighbor_edges
                    .into_iter()
                    .map(|bond| match bond.weight() {
                        Bond::Single => 1,
                        Bond::Double => 2,
                        _ => panic!("Can't handle this bond type yet"),
                    })
                    .sum();
                let needed_hydrogen = desired_bonds_num - current_bonds_num;
                for _ in 0..needed_hydrogen {
                    let new_atom_idx = graph.add_node(Atom::Element(Element::Hydrogen));
                    graph.add_edge(atom_index, new_atom_idx, Bond::Single);
                }
            }
        }

        add_chain_to_graph(&mut graph, &chain, None, None);
        fill_graph_with_hydrogen(&mut graph);

        graph
    }

    pub fn find_main_carbon_chain(&self) -> Vec<NodeIndex> {
        let carbon_atoms = NodeFiltered::from_fn(&**self, |node_id| {
            let node = &self[node_id];
            match node {
                Atom::AliphaticOrganic(atom) => atom.element == Element::Carbon,
                _ => false,
            }
        });

        let node_ids = carbon_atoms.node_identifiers();
        let node_pairs = node_ids.permutations(2).collect::<Vec<_>>();

        let all_paths: Vec<_> = node_pairs
            .into_iter()
            .map(|pair| {
                let path = astar(
                    &carbon_atoms,
                    pair[0],
                    |finish| finish == pair[1],
                    |_| 1,
                    |_| 0,
                )
                .unwrap();
                path
            })
            .collect();
        let longest_path = all_paths.into_iter().max_by_key(|n| n.0).unwrap();

        longest_path.1
    }
}
