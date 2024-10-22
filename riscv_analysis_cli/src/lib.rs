pub mod wrapper {
    use riscv_analysis::parser::{Position, Range};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TestCase {
        pub diagnostics: Vec<DiagnosticTestCase>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DiagnosticTestCase {
        pub file: Option<String>,
        pub title: String,
        pub description: String,
        pub level: String,
        pub range: RangeTestCase,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct RangeTestCase {
        pub start: PositionTestCase,
        pub end: PositionTestCase,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct PositionTestCase {
        pub line: usize,
        pub column: usize,
        pub raw: usize,
    }

    impl From<Position> for PositionTestCase {
        fn from(value: Position) -> Self {
            Self {
                line: value.line,
                column: value.column,
                raw: value.raw_index,
            }
        }
    }

    impl From<Range> for RangeTestCase {
        fn from(value: Range) -> Self {
            Self {
                start: value.start.into(),
                end: value.end.into(),
            }
        }
    }
}
