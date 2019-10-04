use enum_map::EnumMap;

#[derive(Debug, Enum, Clone, Copy)]
pub enum Molecule {
    X, // CO2
    W, // H2O, water
    G, // Glucose
    O, // Oxygen
    A, // ATP
    S, // Starch, long term storage, less dense, more efficient
    F, // Fat, long term storage, more dense, less efficient
    B, // Barrier for cell wall
    I, // Instruction for gene
}

struct MoleculeInfo {
    name: &'static str,
    maximum: i64,
}

fn molecule_infos() -> EnumMap<Molecule, MoleculeInfo> {
    return enum_map! {
        Molecule::X => MoleculeInfo { name: "X", maximum: 100000 },
        Molecule::W => MoleculeInfo { name: "W", maximum: 100000 },
        Molecule::G => MoleculeInfo { name: "G", maximum: 10000 },
        Molecule::O => MoleculeInfo { name: "O", maximum: 100000 },
        Molecule::A => MoleculeInfo { name: "A", maximum: 1000 },
        Molecule::S => MoleculeInfo { name: "S", maximum: 1000 },
        Molecule::F => MoleculeInfo { name: "F", maximum: 1000 },
        Molecule::B => MoleculeInfo { name: "B", maximum: 1000 },
        Molecule::I => MoleculeInfo { name: "I", maximum: 1000 },
    };
}

type Change = i64;

struct Reaction {
    changes: EnumMap<Molecule, Change>,
}

fn photosynthesis() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::X => -6,
            Molecule::W => -6,
            Molecule::G => 1,
            Molecule::O => 6,
            _ => 0
        },
    };
}

fn respiration() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::X => 6,
            Molecule::W => 6,
            Molecule::G => -1,
            Molecule::O => -6,
            Molecule::A => 38,
            _ => 0
        },
    };
}

fn gen_starch() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => -100,
            Molecule::A => -100,
            Molecule::S => 1,
            _ => 0
        },
    };
}

fn lys_starch() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => 100,
            Molecule::S => 1,
            _ => 0
        },
    };
}

fn gen_fat() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => -200,
            Molecule::A => -400,
            Molecule::F => 1,
            _ => 0
        },
    };
}

fn lys_fat() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => 200,
            Molecule::F => -1,
            _ => 0
        },
    };
}

fn gen_barrier() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => -2,
            Molecule::A => -10,
            Molecule::B => 1,
            _ => 0
        },
    };
}

fn lys_barrier() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => 2,
            Molecule::A => -10,
            Molecule::B => -1,
            _ => 0
        },
    };
}

fn gen_instruction() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => -2,
            Molecule::A => -10,
            Molecule::I => 1,
            _ => 0
        },
    };
}

fn lys_instruction() -> Reaction {
    return Reaction {
        changes: enum_map! {
            Molecule::G => 2,
            Molecule::A => -10,
            Molecule::I => -1,
            _ => 0
        },
    };
}

pub struct Pool {
    molecule_amounts: EnumMap<Molecule, i64>,
    molecule_infos: EnumMap<Molecule, MoleculeInfo>,
}

impl Pool {
    fn can_apply(&self, reaction: &Reaction) -> bool {
        for (molecule, &change) in reaction.changes.iter() {
            if change > 0 {
                if (self.molecule_amounts[molecule] + change)
                    > self.molecule_infos[molecule].maximum
                {
                    return false;
                }
            } else {
                if change < 0 && -change > self.molecule_amounts[molecule] {
                    return false;
                }
            }
        }
        return true;
    }

    fn apply(&mut self, reaction: &Reaction) -> bool {
        if !self.can_apply(reaction) {
            return false;
        }
        for (molecule, change) in reaction.changes.iter() {
            self.molecule_amounts[molecule] += change;
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply() {
        let mut pool = Pool {
            molecule_amounts: enum_map! {
                Molecule::X => 10,
                Molecule::W => 10,
                Molecule::G => 0,
                Molecule::O => 0,
                Molecule::A => 0,
                _ => 0
            },
            molecule_infos: molecule_infos(),
        };
        let r = pool.apply(&photosynthesis());
        assert_eq!(r, true);
        assert_eq!(pool.molecule_amounts[Molecule::X], 4);
        assert_eq!(pool.molecule_amounts[Molecule::W], 4);
        assert_eq!(pool.molecule_amounts[Molecule::G], 1);
        assert_eq!(pool.molecule_amounts[Molecule::O], 6);
    }

    #[test]
    fn test_apply_insufficient() {
        let mut pool = Pool {
            molecule_amounts: enum_map! {
                Molecule::X => 5,
                Molecule::W => 5,
                Molecule::G => 0,
                Molecule::O => 0,
                Molecule::A => 0,
                _ => 0
            },
            molecule_infos: molecule_infos(),
        };
        let r = pool.apply(&photosynthesis());
        assert_eq!(r, false);
        assert_eq!(pool.molecule_amounts[Molecule::X], 5);
        assert_eq!(pool.molecule_amounts[Molecule::W], 5);
        assert_eq!(pool.molecule_amounts[Molecule::G], 0);
        assert_eq!(pool.molecule_amounts[Molecule::O], 0);
    }

    #[test]
    fn test_apply_one_insufficient() {
        let mut pool = Pool {
            molecule_amounts: enum_map! {
                Molecule::X => 10,
                Molecule::W => 5,
                Molecule::G => 0,
                Molecule::O => 0,
                Molecule::A => 0,
                _ => 0
            },
            molecule_infos: molecule_infos(),
        };
        let r = pool.apply(&photosynthesis());
        assert_eq!(r, false);
        assert_eq!(pool.molecule_amounts[Molecule::X], 10);
        assert_eq!(pool.molecule_amounts[Molecule::W], 5);
        assert_eq!(pool.molecule_amounts[Molecule::G], 0);
        assert_eq!(pool.molecule_amounts[Molecule::O], 0);
    }

    #[test]
    fn test_apply_too_much() {
        let mut pool = Pool {
            molecule_amounts: enum_map! {
                Molecule::X => 10,
                Molecule::W => 10,
                Molecule::G => 20000,
                Molecule::O => 0,
                Molecule::A => 0,
                _ => 0
            },
            molecule_infos: molecule_infos(),
        };
        let r = pool.apply(&photosynthesis());
        assert_eq!(r, false);
        assert_eq!(pool.molecule_amounts[Molecule::X], 10);
        assert_eq!(pool.molecule_amounts[Molecule::W], 10);
        assert_eq!(pool.molecule_amounts[Molecule::G], 20000);
        assert_eq!(pool.molecule_amounts[Molecule::O], 0);
    }

}
