pub enum Phase {
    WinterBuilds,
    FallRetreats,
    FallMoves,
    SpringRetreats,
    SpringMoves
}

//     def __str__(self):
//         return self.name


impl Phase {
    fn get(&self, name: &str) -> Phase {
        match name {
            "Spring Moves" => Phase::SpringMoves,
            "Spring Retreats" => Phase::SpringRetreats,
            "Fall Moves" => Phase::FallMoves,
            "Fall Retreats" => Phase::FallRetreats,
            "Winter Builds" => Phase::WinterBuilds,
            _ => panic!("Unknown phase {}", name)
        }
    }

    pub fn initial() -> Phase {
        Self::SpringMoves
    }

    fn next(&self) -> Phase {
        match self {
            Phase::WinterBuilds => Phase::SpringMoves,
            Phase::FallRetreats => Phase::WinterBuilds,
            Phase::FallMoves => Phase::FallRetreats,
            Phase::SpringRetreats => Phase::FallMoves,
            Phase::SpringMoves => Phase::SpringRetreats,
        }
    }

    fn previous(&self) -> Phase {
        match self {
            Phase::WinterBuilds => Phase::FallRetreats,
            Phase::FallRetreats => Phase::FallMoves,
            Phase::FallMoves => Phase::SpringRetreats,
            Phase::SpringRetreats => Phase::SpringMoves,
            Phase::SpringMoves => Phase::WinterBuilds,
        }
    }

    fn is_moves(&self) -> bool {
        match self {
            Phase::SpringMoves | Phase::FallMoves => true,
            _ => false
        }
    }

    fn is_retreats(&self) -> bool {
        match self {
            Phase::SpringRetreats | Phase::FallRetreats => true,
            _ => false
        }
    }

    fn is_builds(&self) -> bool {
        match self {
            Phase::WinterBuilds => true,
            _ => false
        }
    }
}


