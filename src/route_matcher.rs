use std::collections::{HashMap, VecDeque};

pub type PathParams = HashMap<String, String>;

#[derive(Default, Debug)]
pub struct Node {
    // Given a path /foo/bar value represents one of the entries, "foo" or "bar"
    pub(crate) value: String,
    // If this is a path param we should mark that
    pub(crate) is_param: bool,
    // If this is the last segment in a path we should mark that to know when to stop
    // looking for children.
    pub(crate) is_terminal: bool,
    // Given a path /foo/bar, when represented in the Trie structure "bar" is a child of "foo"
    pub(crate) children: HashMap<String, Node>,
    // When the node is a wildcard node or terminal we should store the url pattern it maps to
    pub(crate) url: Option<String>,
    // A wildcard node is a node that accepts any url segments below itself.
    pub(crate) is_wildcard: bool,
}

impl Node {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            value: "".into(),
            url: None,
            is_param: false,
            is_terminal: false,
            is_wildcard: false,
            children: HashMap::new(),
        }
    }

    pub fn append(&mut self, url_path_pattern: &str) {
        let url_path_pattern = url_path_pattern.trim().to_lowercase();
        let mut url_parts = url_path_pattern.split('/').collect::<VecDeque<&str>>();
        if let Some(first) = url_parts.front() {
            if first.is_empty() {
                url_parts.pop_front();
            }
        } else {
            return;
        }

        let mut node_ref = self;
        while let Some(segment) = url_parts.pop_front() {
            let segment = segment.trim();
            let is_terminal = url_parts.is_empty();
            let is_wildcard = if !is_terminal && url_parts.iter().next().unwrap().eq(&"*") {
                true
            } else {
                false
            };
            node_ref = node_ref
                .children
                .entry(segment.to_string())
                .or_insert(Node {
                    value: segment.to_owned(),
                    url: if is_terminal || is_wildcard {
                        Some(url_path_pattern.clone())
                    } else {
                        None
                    },
                    is_param: segment.starts_with(':'),
                    is_terminal,
                    is_wildcard,
                    children: HashMap::new(),
                });
            if is_wildcard {
                break;
            }
        }
    }

    fn get_path_param_from_children(&self) -> Option<&Node> {
        self.children
            .iter()
            .find_map(|(_, node)| if node.is_param { Some(node) } else { None })
    }

    pub fn find_match(&self, url_string: &str) -> Option<(String, PathParams)> {
        let url_string = url_string.trim().to_lowercase();
        if self.children.is_empty() && !url_string.is_empty() {
            return None;
        }

        if !self.children.is_empty() && url_string.is_empty() {
            return None;
        }

        let mut url_parts = url_string.split("/").collect::<VecDeque<&str>>();

        if let Some(first) = url_parts.front() {
            if first.is_empty() {
                url_parts.pop_front();
            }
        } else {
            return None;
        }

        let mut node_ref = self;
        let mut path_params: PathParams = PathParams::new();
        while let Some(segment) = url_parts.pop_front() {
            if let Some(node) = node_ref.children.get(segment.trim()) {
                if node.is_terminal && url_parts.is_empty() {
                    return Some((node.url.clone().unwrap(), path_params));
                }
                if node.is_wildcard {
                    return Some((node.url.clone().unwrap(), path_params));
                }
                node_ref = node;
            } else if let Some(node) = node_ref.get_path_param_from_children() {
                path_params.insert(node.value.replace(':', ""), segment.to_string());
                if node.is_terminal && url_parts.is_empty() {
                    return Some((node.url.clone().unwrap(), path_params));
                }
                node_ref = node;
            } else {
                return None;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::route_matcher::PathParams;

    use super::Node;

    #[test]
    fn strips_leading_slash_in_url_pattern() {
        let mut node = Node::new();
        node.append("/foo");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.value, String::new());
        assert_eq!(node.children.get("foo").unwrap().value, "foo".to_string());
    }

    #[test]
    fn ignores_accidental_spaces_in_url() {
        let mut node = Node::new();
        node.append("   /foo");
        assert_eq!(node.children.len(), 1);
        assert!(node.children.contains_key("foo"));

        node = Node::new();
        node.append("/bar   /   baz/   foo");

        let assert_child_exists = |node: &Node, key: &str| {
            let child = node.children.get(key);
            assert!(child.is_some());
            assert_eq!(child.unwrap().value, key);
        };

        assert_child_exists(&node, "bar");
        assert_child_exists(node.children.get("bar").unwrap(), "baz");
        assert_child_exists(
            node.children
                .get("bar")
                .unwrap()
                .children
                .get("baz")
                .unwrap(),
            "foo",
        );
    }

    #[test]
    fn creates_a_tree_from_urls_without_path_params() {
        let mut node = Node::new();
        node.append("foo/bar/baz");
        assert_eq!(node.children.len(), 1);
        let child = node.children.get("foo");
        assert!(child.is_some());
        let child = child.unwrap().children.get("bar");
        assert!(child.is_some());
        let child = child.unwrap().children.get("baz");
        assert!(child.is_some());
        assert!(child.unwrap().is_terminal);

        node.append("foo/boo");
        assert_eq!(node.children.len(), 1);
        let child = node.children.get("foo").unwrap();
        assert_eq!(child.children.len(), 2);
        assert!(child.children.contains_key("boo"));
    }

    #[test]
    fn matches_simple_urls() {
        let mut node = Node::new();
        node.append("/foo/bar/baz");
        assert_eq!(
            node.find_match("/foo/bar/baz"),
            Some(("/foo/bar/baz".into(), PathParams::new()))
        );
        assert_eq!(node.find_match("/foo/baz/bar"), None);
        assert_eq!(node.find_match("/foo/bar/baz/taz"), None);
        assert_eq!(
            node.find_match("/  foo/ bar  /baz"),
            Some(("/foo/bar/baz".into(), PathParams::new()))
        );
    }

    #[test]
    fn appends_urls_with_path_params() {
        let mut node = Node::new();
        node.append("/foo/:id/bar/:id");
        let check_node = |node: &Node, key: &str, is_param: bool| {
            let child = node.children.get(key);
            assert!(child.is_some());
            assert!(child.unwrap().value.eq(key));
            assert_eq!(child.unwrap().is_param, is_param);
        };

        check_node(&node, "foo", false);
        let child = node.children.get("foo").unwrap();
        check_node(child, ":id", true);
        let child = child.children.get(":id").unwrap();
        check_node(child, "bar", false);
        let child = child.children.get("bar").unwrap();
        check_node(child, ":id", true);
    }

    #[test]
    fn match_path_params() {
        let mut node = Node::new();
        // simple param matching
        node.append("/echo/:param");
        assert!(node.find_match("/echo/foo").is_some());
        assert!(node.find_match("/echo/blabla/foo").is_none());

        // multi-level param matching
        node.append("/echo/blabla/:param");
        assert!(node.find_match("/echo/blabla/foo").is_some());
        let (pattern, params) = node.find_match("/echo/blabla/foo").unwrap();
        assert_eq!(pattern, "/echo/blabla/:param".to_string());
        assert_eq!(params, PathParams::from([("param".into(), "foo".into())]));

        // If there are 2 patterns that could match a url,
        // the affinity is towards an exact match pattern, rather than a pattern with a param.
        node.append("/echo/blabla/foo");
        let (pattern, params) = node.find_match("/echo/blabla/foo").unwrap();
        assert_eq!(pattern, "/echo/blabla/foo".to_string());
        assert_eq!(params, PathParams::new());
    }

    #[test]
    fn matches_index_handler_when_specified() {
        let mut node = Node::new();
        node.append("/");
        node.append("/path");
        node.append("/some/:other/path");

        let index_match = node.find_match("/");
        assert!(index_match.is_some());
        let (path, params) = index_match.unwrap();
        assert_eq!(path, "/");
        assert_eq!(params.len(), 0);

        let path_match = node.find_match("/path");
        assert!(path_match.is_some());
        let (path, params) = path_match.unwrap();
        assert_eq!(path, "/path");
        assert_eq!(params.len(), 0);

        let some_other_path_match = node.find_match("/some/foo/path");
        assert!(some_other_path_match.is_some());
        let (path, params) = some_other_path_match.unwrap();
        assert_eq!(path, "/some/:other/path");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn does_not_matches_index_handler_when_not_specified() {
        let mut node = Node::new();
        node.append("/path");

        let index_match = node.find_match("/");
        assert!(index_match.is_none());

        let path_match = node.find_match("/path");
        assert!(path_match.is_some());
        let (path, params) = path_match.unwrap();
        assert_eq!(path, "/path");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn matches_star_in_pathnames() {
        let mut node = Node::new();
        node.append("/static/*");

        let find_result = node.find_match("/static");
        assert!(find_result.is_some());
        let (pattern, _) = find_result.unwrap();
        assert_eq!(pattern, "/static/*");

        let find_result = node.find_match("/static/index.html");
        assert!(find_result.is_some());
        let (pattern, _) = find_result.unwrap();
        assert_eq!(pattern, "/static/*");

        let find_result = node.find_match("/static/assets/image.jpg");
        assert!(find_result.is_some());
        let (pattern, _) = find_result.unwrap();
        assert_eq!(pattern, "/static/*");
    }

    #[test]
    fn matcher_respects_casing() {}
}
