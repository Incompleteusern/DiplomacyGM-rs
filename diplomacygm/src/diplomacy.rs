pub mod persistence {
    pub mod board;
    pub mod manager;

    pub mod player;
    pub mod province;
    pub mod unit;
    pub mod order;
    pub mod phase;
}

pub mod map_parser {
    pub mod vector {
        pub mod vector;
        pub mod utils;
        pub mod transform;
    }
}

pub mod config_parser {
    pub mod config;
}