#[cfg(test)]
mod tests {
    use crate::parser::{ParserNode, RVStringParser};

    #[test]
    fn can_parse_comments_without_errors() {
        let (nodes, errors) =
            RVStringParser::parse_from_text("add x1, x10, x11 #this is my comment\n");
        assert_eq!(nodes.len(), 2);
        assert_eq!(errors.len(), 0);
        matches!(&nodes[0], ParserNode::ProgramEntry(_));
        matches!(&nodes[1], ParserNode::Arith(_));
        assert_eq!(nodes[1].to_string(), "add ra <- a0, a1");
    }
}
