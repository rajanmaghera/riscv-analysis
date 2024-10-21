pub mod wrapper {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ListWrapper {
        pub diagnostics: Vec<DiagnosticWrapper>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DiagnosticWrapper {
        pub file: Option<String>,
        pub title: String,
        pub description: String,
        pub level: String,
        pub range: RangeWrapper,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct RangeWrapper {
        pub start: PositionWrapper,
        pub end: PositionWrapper,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct PositionWrapper {
        pub line: usize,
        pub column: usize,
        pub raw: usize,
    }

}
