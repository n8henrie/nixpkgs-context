use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use tree_sitter::{Node, Parser, Tree};

use crate::{Error, Result};

fn find_binding(node: Node) -> Option<Node> {
    match node.kind() {
        "binding" => Some(node),
        // skip e.g. callPackage parameters
        "function_expression" => None,
        _ => node.parent().and_then(find_binding),
    }
}

fn node_text<'a>(source_code: &'a str, node: Node) -> &'a str {
    &source_code[node.byte_range()]
}

#[derive(Debug)]
pub(crate) struct Context {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) code: String,
}

impl Context {
    fn try_from_source(
        tree: &Tree,
        source_code: impl AsRef<str>,
        idx: usize,
        needle: impl AsRef<str>,
        entry: impl AsRef<Path> + Clone,
    ) -> Option<Self> {
        let source_code = source_code.as_ref();
        let needle = needle.as_ref();
        let needle_length = needle.len();
        let node = tree
            .root_node()
            .descendant_for_byte_range(idx, idx + needle_length)?;
        // Reject partial matches
        if node.byte_range() != (idx..idx + needle.len()) {
            return None;
        }
        let binding = find_binding(node)?;
        let code = node_text(source_code, binding).to_string();
        let name = if let Some(attrpath) = binding.child_by_field_name("attrpath") {
            node_text(source_code, attrpath).to_string()
        } else {
            "<empty>".to_string()
        };
        Some(Context {
            name,
            code,
            path: entry.as_ref().to_path_buf(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ContextVec(Vec<Context>);

impl ContextVec {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn try_from_source(
        source_code: impl AsRef<str>,
        parser: &mut Parser,
        needle: impl AsRef<str>,
        entry: impl AsRef<Path> + Clone,
    ) -> Result<Self> {
        let source_code = source_code.as_ref();
        let needle = needle.as_ref();
        let indices: Vec<_> = source_code
            .match_indices(needle)
            .map(|(idx, _)| idx)
            .collect();
        if indices.is_empty() {
            return Ok(ContextVec::new());
        }
        let Some(tree) = parser.parse(source_code, None) else {
            return Err(Error::Parse(entry.as_ref().to_path_buf()));
        };
        let mut vec = Vec::with_capacity(indices.len());
        for idx in indices {
            if let Some(ctx) = Context::try_from_source(&tree, source_code, idx, needle, &entry) {
                vec.push(ctx);
            }
        }
        Ok(Self(vec))
    }

    pub(crate) fn push(&mut self, ctx: Context) {
        self.0.push(ctx);
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Context> {
        self.0.iter()
    }
}

impl IntoIterator for ContextVec {
    type Item = Context;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Ord for ContextVec {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.len().cmp(&other.0.len())
    }
}

impl PartialOrd for ContextVec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ContextVec {}
impl PartialEq for ContextVec {
    fn eq(&self, other: &Self) -> bool {
        self.0.len().eq(&other.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_nix::LANGUAGE.into())
            .expect("error loading nix grammar");
        parser
    }

    #[test]
    fn test_find_binding() {
        let source = "{ foo = ''bar''; }";
        let tree = parser().parse(source, None).unwrap();

        let needle = "bar";
        let start = source.find(needle).unwrap();
        let end = start + needle.len();
        let node = tree
            .root_node()
            .descendant_for_byte_range(start, end)
            .unwrap();
        assert!(find_binding(node).is_some());
        assert_eq!(
            find_binding(node)
                .unwrap()
                .child_by_field_name("attrpath")
                .unwrap()
                .child_by_field_name("attr")
                .unwrap()
                .kind(),
            "identifier"
        );
    }

    #[test]
    fn find_binding_skips_callpackage_args() {
        let source = "{lib, stdenv, fakePkg}: stdenv.mkDerivation { buildInputs = [ fakePkg ]; }";
        let tree = parser().parse(source, None).unwrap();

        let needle = "fakePkg";

        let bindings: Vec<_> = source
            .match_indices(needle)
            .map(|(idx, _)| {
                let end = idx + needle.len();
                let node = tree
                    .root_node()
                    .descendant_for_byte_range(idx, end)
                    .unwrap();
                find_binding(node)
            })
            .collect();
        assert!(&bindings[0].is_none());
        assert!(&bindings[1].is_some());
        assert_eq!(
            source.find("buildInputs").unwrap(),
            bindings[1].unwrap().start_byte()
        );
    }

    #[test]
    fn test_node_text() {
        let source = "{lib, stdenv, fakePkg}: stdenv.mkDerivation { buildInputs = [ fakePkg ]; }";
        let tree = parser().parse(source, None).unwrap();
        let text = node_text(source, tree.root_node());
        assert_eq!(text, source);

        let needle = "fakePkg";
        // rfind so we can skip the callPackage args
        let start = source.rfind(needle).unwrap();
        let end = start + needle.len();
        let node = find_binding(
            tree.root_node()
                .descendant_for_byte_range(start, end)
                .unwrap(),
        )
        .unwrap();
        let text = node_text(source, node);
        assert_eq!(text, "buildInputs = [ fakePkg ];");
    }
}
