use crate::dfa::{Dfa, DfaState};
use crate::nfa::{Nfa, NfaState};
use crate::*;
use ::regex::Regex as LibRegex;
use proptest::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::rc::Rc;

struct MultipleCounterIter {
    state: Vec<usize>,
    len: usize,
    max_number: usize,
    has_returned: bool,
    is_finished: bool,
}

impl MultipleCounterIter {
    fn new(length: usize, max: usize) -> Self {
        MultipleCounterIter {
            state: vec![0; length],
            len: 0,
            max_number: max,
            has_returned: false,
            is_finished: false,
        }
    }
}

impl Iterator for MultipleCounterIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        if !self.has_returned {
            self.has_returned = true;
            return Some(vec![]);
        }

        for i in (0..self.len).rev() {
            self.state[i] += 1;
            if self.state[i] > self.max_number {
                self.state[i] = 0;
            } else {
                return Some(self.state[0..self.len].to_vec());
            }
        }

        self.len += 1;

        if self.len > self.state.len() {
            self.is_finished = true;
            None
        } else {
            Some(self.state[0..self.len].to_vec())
        }
    }
}

#[test]
fn test_subset_construction() {
    let dfa_source = include_str!("../tests/test_files/eq_to_nfa1.dfa");
    let parsed_dfa = parser::dfa(dfa_source).unwrap();
    let dfa: Dfa = parsed_dfa.try_into().unwrap();

    let nfa_source = include_str!("../tests/test_files/nfa1.nfa");
    let parsed_nfa = parser::nfa(nfa_source).unwrap();
    let nfa: Nfa = parsed_nfa.try_into().unwrap();

    let converted = nfa.to_dfa();
    assert!(dfa.equivalent_to(&converted));
}

proptest! {
    /// Tests that a DFA can be turned into a table with dfa.to_table() and then be
    /// parsed to the *very same* DFA again (not just equivalent)
    #[test]
    fn dfa_table_reparse(dfa in dfa(50, 50)) {
        let parsed_dfa: Dfa = parser::dfa(&dfa.to_table()).unwrap().try_into().unwrap();
        assert_eq!(dfa, parsed_dfa);
    }

    /// Tests that a DFA can be minimized and is then still equivalent to the original DFA
    #[test]
    fn dfa_minimize_eq(dfa in dfa(25, 25)) { // This size is adequate, larger size takes too long time
        let mut minimized_dfa = dfa.clone();
        minimized_dfa.minimize();
        assert!(minimized_dfa.equivalent_to(&dfa), "Minimized DFA should be equivalent to original");
        assert!(dfa.equivalent_to(&minimized_dfa), "Original DFA should be equivalent to original");
    }

    /// Tests that a DFA can be turned into an NFA and then turned back again to a DFA
    /// while still being equivalent to the original DFA
    #[test]
    fn dfa_to_nfa_to_dfa(dfa in dfa(50, 50)) {
        let converted = dfa.clone().to_nfa().to_dfa();
        assert!(dfa.equivalent_to(&converted), "DFA should be equivalent to DFA->NFA->DFA");
        assert!(converted.equivalent_to(&dfa), "DFA->NFA->DFA should be equivalent to DFA");
    }


    /// Tests that a NFA can be turned into a table with dfa.to_table() and then be
    /// parsed to the *very same* DFA again (not just equivalent)
    #[test]
    fn nfa_table_reparse(nfa in nfa(50, 50)) {
        let parsed_nfa: Nfa = parser::nfa(&nfa.to_table()).unwrap().try_into().unwrap();
        assert_eq!(nfa, parsed_nfa);
    }

    /// Tests that a NFA can be turned into an DFA and then turned back again to a NFA
    /// while still being equivalent to the original NFA
    #[test]
    fn nfa_to_dfa_to_nfa(nfa in nfa(25, 25)) {
        let converted = nfa.to_dfa().to_nfa();
        assert!(nfa.equivalent_to(&converted), "NFA should be equivalent to NFA->DFA->NFA");
        assert!(converted.equivalent_to(&nfa), "NFA->DFA->NFA should be equivalent to NFA");
    }

    #[test]
    fn dfa_binary_ops(
        dfa1 in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        dfa2 in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        tests in prop::collection::vec("[a-f]+", 100)
    ) {
        let intersection = dfa1.intersection(&dfa2).unwrap();
        let union = dfa1.union(&dfa2).unwrap();
        let difference = dfa1.difference(&dfa2).unwrap();
        let symmetric_difference = dfa1.symmetric_difference(&dfa2).unwrap();
        for test in tests.iter() {
            let r1 = dfa1.accepts_graphemes(test);
            let r2 = dfa2.accepts_graphemes(test);
            assert_eq!(intersection.accepts_graphemes(test), r1 && r2);
            assert_eq!(union.accepts_graphemes(test), r1 || r2);
            assert_eq!(difference.accepts_graphemes(test), r1 && !r2);
            assert_eq!(symmetric_difference.accepts_graphemes(test), r1 != r2);
        }
    }

    #[test]
    fn nfa_binary_ops(
        // This takes a really long time to run, so we reduce the size of NFAs tested and amount of test cases
        nfa1 in fixed_alphabet_nfa(8, 'a'..='f', ('a'..='f').count()),
        nfa2 in fixed_alphabet_nfa(8, 'a'..='f', ('a'..='f').count()),
        tests in prop::collection::vec("[a-f]+", 50)
    ) {
        let intersection = nfa1.intersection(&nfa2).unwrap();
        let union = nfa1.clone().union(nfa2.clone()).unwrap();
        for test in tests.iter() {
            let r1 = nfa1.accepts_graphemes(test);
            let r2 = nfa2.accepts_graphemes(test);
            assert_eq!(intersection.accepts_graphemes(test), r1 && r2);
            assert_eq!(union.accepts_graphemes(test), r1 || r2);
        }
    }

    #[test]
    fn dfa_self_union(dfa in fixed_alphabet_dfa(20, 'a'..='z', ('a'..='z').count())) {
        let union = dfa.union(&dfa).unwrap();
        assert!(union.equivalent_to(&dfa));
    }

    #[test]
    fn dfa_self_intersection(dfa in fixed_alphabet_dfa(20, 'a'..='z', ('a'..='z').count())) {
        let intersection = dfa.intersection(&dfa).unwrap();
        assert!(intersection.equivalent_to(&dfa));
    }

    #[test]
    fn dfa_inversion_tautologies(
        dfa in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        tests in prop::collection::vec("[a-f]+", 100)
    ) {
        // dfa OR !dfa should accept everything
        // dfa AND !dfa should accept nothing
        let inv_dfa = {
            let mut dfa = dfa.clone();
            dfa.invert();
            dfa
        };
        let union = dfa.union(&inv_dfa).unwrap();
        let intersection = dfa.intersection(&inv_dfa).unwrap();
        assert!(union.has_reachable_accepting_state());
        assert!(!intersection.has_reachable_accepting_state());
        tests.iter().for_each(|test| {
            assert!(union.accepts_graphemes(test));
            assert!(!intersection.accepts_graphemes(test));
        });
    }

    #[test]
    fn nfa_remove_epsilon_transitions(
        nfa in nfa(25, 25)
    ) {
        let mut no_eps = nfa.clone();
        no_eps.remove_epsilon_moves();
        assert!(nfa.equivalent_to(&no_eps));
        assert!(no_eps.states().iter().all(|s| s.epsilon_transitions.is_empty()));
        assert!(!no_eps.has_epsilon_moves());
    }

    #[test]
    fn nfa_remove_unreachable_states(
        nfa in nfa(25, 25)
    ) {
        let mut no_unr_states = nfa.clone();
        no_unr_states.remove_unreachable_states();
        assert!(nfa.equivalent_to(&no_unr_states));
    }

    #[test]
    fn nfa_words(
        dfa in fixed_alphabet_dfa(25, 'a'..='f', ('a'..='f').count())
    ) {
        let nfa = dfa.to_nfa();
        let mut no_eps = nfa.clone();
        no_eps.remove_epsilon_moves();

        let inverse = {
            let mut dfa = nfa.to_dfa();
            dfa.minimize();
            dfa.invert();
            let mut nfa = dfa.to_nfa();
            nfa.remove_epsilon_moves();
            nfa
        };

        no_eps.words().take(100).for_each(|word|{
            assert!(nfa.accepts_graphemes(&word));
            assert!(!inverse.accepts_graphemes(&word));
        });

        inverse.words().take(100).for_each(|word| {
            assert!(!nfa.accepts_graphemes(&word));
            assert!(inverse.accepts_graphemes(&word));
        });

        // This checks that merging both iter_nfa and iter_inv gives all words
        // and in lexicographic order. It only checks for words up to length 3
        // due to exponential growth. This also checks that the iterators doesn't
        // "skip" words or generates duplicate words since all words should be in
        // exactly one of the iterators.
        let mut iter = MultipleCounterIter::new(3, nfa.alphabet().len() - 1);
        let mut iter_nfa = nfa.word_component_indices();
        let mut next_nfa = iter_nfa.next();
        let mut iter_inv = inverse.word_component_indices();
        let mut next_inv = iter_inv.next();
        while let Some(word) = iter.next() {
            if next_nfa.as_ref().map_or(false, |w| w == &word) {
                next_nfa = iter_nfa.next();
            } else if next_inv.as_ref().map_or(false, |w| w == &word) {
                next_inv = iter_inv.next();
            } else {
                panic!("Missed component sequence {word:?}");
            }
        }
    }

    #[test]
    fn regex(
        regex_str in random_regex("[a-z]"),
        tests in prop::collection::vec("[a-z]+", 20)
    ) {
        let regex = parser::regex(&regex_str).unwrap();
        let mut dfa = regex.to_nfa().to_dfa();
        dfa.minimize();
        let lib_regex = LibRegex::new(&format!("^({regex_str})$")).unwrap();

        let accepted_chars = regex_str.chars().collect::<HashSet<_>>();

        tests.iter().for_each(|test|{
            // Need to filter string since it can't use characters not in the regex itself
            // due to the DFA alphabet
            let s = test.chars().filter(|c| accepted_chars.contains(c)).collect::<String>();
            assert_eq!(dfa.accepts_graphemes(&s), lib_regex.is_match(&s));
        })
    }

    #[test]
    fn regex_parse(regex_str in random_regex("[a-zε∅]")) {
        let parse1 = parser::regex(&regex_str).unwrap();
        let stringified = parse1.to_string();
        let parse2 = parser::regex(&stringified).unwrap();
        assert!(parse1.to_nfa().equivalent_to(&parse2.to_nfa()));
    }
}

prop_compose! {
    fn fixed_alphabet_nfa(max_states: usize, alphabet: RangeInclusive<char>, alphabet_size: usize)
        (num_states in 1..max_states)
        (
            states in state_names(num_states),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            transitions in prop::collection::vec(nfa_transitions(num_states, alphabet_size), num_states..=num_states),
            epsilon_transitions in prop::collection::vec(epsilon_transitions(num_states), num_states..=num_states),
        )
    -> Nfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter().zip(
                    epsilon_transitions.into_iter()
                )
            )
        ).enumerate().map(|(idx, (state_name, (accepting, (transitions, epsilon_transitions))))|
            NfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                transitions,
                epsilon_transitions
            }
        ).collect();

        let mut alphabet: Vec<Rc<str>> = alphabet.clone().map(|c| Rc::from(c.to_string())).collect();
        alphabet.shuffle(&mut thread_rng());
        let alphabet = Rc::from(alphabet);

        Nfa {
            alphabet,
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn nfa(max_states: usize, max_alphabet_size: usize)
        (num_states in 1..max_states, alphabet_size in 1..max_alphabet_size)
        (
            states in state_names(num_states),
            alphabet in alphabet_elems(alphabet_size),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            epsilon_transitions in prop::collection::vec(epsilon_transitions(num_states), num_states..=num_states),
            transitions in prop::collection::vec(nfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Nfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter().zip(
                    epsilon_transitions.into_iter()
                )
            )
        ).enumerate().map(|(idx, (state_name, (accepting, (transitions, epsilon_transitions))))|
            NfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                epsilon_transitions,
                transitions
            }
        ).collect();

        Nfa {
            alphabet: alphabet.iter().map(|entry| Rc::from(entry.as_str())).collect(),
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn fixed_alphabet_dfa(max_states: usize, alphabet: RangeInclusive<char>, alphabet_size: usize)
        (num_states in 1..max_states)
        (
            states in state_names(num_states),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            transitions in prop::collection::vec(dfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Dfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter()
            )
        ).enumerate().map(|(idx, (state_name, (accepting, transitions)))|
            DfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                transitions
            }
        ).collect();

        let mut alphabet: Vec<Rc<str>> = alphabet.clone().map(|c| Rc::from(c.to_string())).collect();
        alphabet.shuffle(&mut thread_rng());
        let alphabet = Rc::from(alphabet);

        Dfa {
            alphabet,
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn dfa(max_states: usize, max_alphabet_size: usize)
        (num_states in 1..max_states, alphabet_size in 1..max_alphabet_size)
        (
            states in state_names(num_states),
            alphabet in alphabet_elems(alphabet_size),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            transitions in prop::collection::vec(dfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Dfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter()
            )
        ).enumerate().map(|(idx, (state_name, (accepting, transitions)))|
            DfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                transitions
            }
        ).collect();

        Dfa {
            alphabet: alphabet.iter().map(|entry| Rc::from(entry.as_str())).collect(),
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn dfa_transitions(states: usize, alphabet_size: usize)
        (transitions in prop::collection::vec(0..states, alphabet_size..=alphabet_size))
    -> Vec<usize> {
        transitions
    }
}

prop_compose! {
    fn epsilon_transitions(states: usize)
        (transitions in prop::collection::vec(any::<bool>(), states..=states))
    -> Vec<usize> {
        let mut rng = thread_rng();
        let mut transitions: Vec<_> = transitions.into_iter()
            .enumerate()
            .filter_map(|(idx, b)| b.then_some(idx))
            .collect();
        transitions.shuffle(&mut rng);
        transitions
    }
}

prop_compose! {
    fn nfa_transitions(states: usize, alphabet_size: usize)
        (transitions in prop::collection::vec(
            // This is a bytevec saying for each state if it has a transition there or not
            // HashMap would be a better fit but maybe too much rejections?
            prop::collection::vec(any::<bool>(), states..=states),
            alphabet_size..=alphabet_size
        ))
    -> Vec<Vec<usize>> {
        let mut rng = thread_rng();
        transitions.into_iter()
            .map(|row| {
                let mut row: Vec<usize> = row.into_iter()
                    .enumerate()
                    .filter_map(|(idx, b)| b.then_some(idx))
                    .collect();
                row.as_mut_slice().shuffle(&mut rng);
                row
            })
            .collect()
    }
}

prop_compose! {
    fn state_names(count: usize)
        (names in filtered_set(count, r"[^\s#{}]+", &["ε", "eps", "→", "->", "*"]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn simple_alphabet(count: usize)
        (names in filtered_set(std::cmp::max(count, 4), "[a-e]", &[]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn alphabet_elems(count: usize)
        (names in filtered_set(count, r"[^\s#{}]+", &["ε", "eps", "→", "->", "*"]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn filtered_set(count: usize, regex: &'static str, deny: &'static [&'static str])
        (names in prop::collection::hash_set(
            regex.prop_filter( // No whitespace
                "name should not be reserved",
                |s| !deny.contains(&s.as_str()) && !s.contains(|c: char| c.is_whitespace())
            ),
            count..=count
        ))
    -> HashSet<String> {
        names
    }
}

fn random_regex(base: &'static str) -> impl Strategy<Value = String> {
    base.prop_recursive(20, 1024, 20, |inner| {
        prop_oneof![
            10 => prop::collection::vec(inner.clone(), 1..20)
                .prop_map(|vec| format!("({})", vec.join(""))),
            10 => prop::collection::vec(inner.clone(), 1..20).prop_map(|vec| vec.join("|")),
            3 => inner.clone().prop_map(|r| format!("({r})*")),
            3 => inner.clone().prop_map(|r| format!("({r})+")),
        ]
    })
}

#[test]
fn test_parse_grammar() {
    let grammar_source = include_str!("../tests/test_files/grammar1.cfg");
    let parsed_grammar = parser::grammar(grammar_source).unwrap();
    let grammar: parser::ParsedGrammar = parsed_grammar.try_into().unwrap();

    let expected_grammar = parser::ParsedGrammar {
        terminals: vec!["1", "+", "-"],
        nonterminals: vec!["E", "N", "O"],
        productions: vec![
            parser::ParsedProduction {
                name: "N",
                alternatives: vec![vec!["1", "N"], vec!["1"]],
            },
            parser::ParsedProduction {
                name: "O",
                alternatives: vec![vec!["+"], vec![], vec!["-"]],
            },
            parser::ParsedProduction {
                name: "E",
                alternatives: vec![vec!["N"], vec!["E", "O", "E"]],
            },
        ],
        start: "E",
    };

    assert!(grammar == expected_grammar);
}
