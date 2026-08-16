//! `nish` shell — parser and AST.
//!
//! Tokens → AST. Precedence: `;`/`&&`/`||` separate pipelines; `|` joins
//! commands; redirections bind to a single command.

use super::lexer::{lex, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirDir {
    In,
    Out,
    Append,
}

/// A single command word plus its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCmd {
    /// Leading `VAR=value` assignments scoped to this command.
    pub assignments: Vec<(String, String)>,
    pub name: String,
    pub args: Vec<String>,
}

/// Command AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Simple(SimpleCmd),
    Pipe(Box<Node>, Box<Node>),
    Redir {
        cmd: Box<Node>,
        dir: RedirDir,
        target: String,
    },
    Sequence(Box<Node>, Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
}

pub fn parse(input: &str) -> Node {
    let tokens = lex(input);
    let mut pos = 0;
    parse_list(&tokens, &mut pos)
}

fn parse_list(tokens: &[Token], pos: &mut usize) -> Node {
    let mut left = parse_and_or(tokens, pos);
    while let Token::Semicolon = &tokens[*pos] {
        *pos += 1;
        let right = parse_and_or(tokens, pos);
        left = Node::Sequence(Box::new(left), Box::new(right));
    }
    left
}

/// `&&` and `||` bind tighter than `;`, matching POSIX precedence.
fn parse_and_or(tokens: &[Token], pos: &mut usize) -> Node {
    let mut left = parse_pipeline(tokens, pos);
    loop {
        match &tokens[*pos] {
            Token::And => {
                *pos += 1;
                let right = parse_pipeline(tokens, pos);
                left = Node::And(Box::new(left), Box::new(right));
            }
            Token::Or => {
                *pos += 1;
                let right = parse_pipeline(tokens, pos);
                left = Node::Or(Box::new(left), Box::new(right));
            }
            _ => break,
        }
    }
    left
}

fn parse_pipeline(tokens: &[Token], pos: &mut usize) -> Node {
    let mut left = parse_command(tokens, pos);
    while tokens[*pos] == Token::Pipe {
        *pos += 1;
        let right = parse_command(tokens, pos);
        left = Node::Pipe(Box::new(left), Box::new(right));
    }
    left
}

fn parse_command(tokens: &[Token], pos: &mut usize) -> Node {
    let mut words: Vec<String> = Vec::new();
    let mut redirs: Vec<(RedirDir, String)> = Vec::new();

    loop {
        match &tokens[*pos] {
            Token::Word(w) => {
                words.push(w.clone());
                *pos += 1;
            }
            Token::RedirIn => {
                *pos += 1;
                if let Token::Word(t) = &tokens[*pos] {
                    redirs.push((RedirDir::In, t.clone()));
                    *pos += 1;
                }
            }
            Token::RedirOut => {
                *pos += 1;
                if let Token::Word(t) = &tokens[*pos] {
                    redirs.push((RedirDir::Out, t.clone()));
                    *pos += 1;
                }
            }
            Token::RedirAppend => {
                *pos += 1;
                if let Token::Word(t) = &tokens[*pos] {
                    redirs.push((RedirDir::Append, t.clone()));
                    *pos += 1;
                }
            }
            _ => break,
        }
    }

    let mut node = Node::Simple(build_simple(words));
    // Apply redirections innermost-out so `cmd > f1 2> f2` semantics stay sane.
    for (dir, target) in redirs.into_iter().rev() {
        node = Node::Redir {
            cmd: Box::new(node),
            dir,
            target,
        };
    }
    node
}

fn build_simple(words: Vec<String>) -> SimpleCmd {
    let mut assignments = Vec::new();
    let mut rest = Vec::new();
    for word in words {
        if rest.is_empty() && is_assignment(&word) {
            let (k, v) = split_assignment(&word);
            assignments.push((k, v));
        } else {
            rest.push(word);
        }
    }
    let name = rest.first().cloned().unwrap_or_default();
    let args = rest.into_iter().skip(1).collect();
    SimpleCmd {
        assignments,
        name,
        args,
    }
}

pub fn is_assignment(word: &str) -> bool {
    if let Some(idx) = word.find('=') {
        if idx == 0 {
            return false;
        }
        let key = &word[..idx];
        key.chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn split_assignment(word: &str) -> (String, String) {
    let idx = word.find('=').unwrap_or(word.len());
    (word[..idx].to_string(), word[idx + 1..].to_string())
}

/// A command line that is empty or only whitespace.
pub fn is_blank(input: &str) -> bool {
    input.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_command() {
        let node = parse("ls -la /home");
        match node {
            Node::Simple(cmd) => {
                assert_eq!(cmd.name, "ls");
                assert_eq!(cmd.args, vec!["-la", "/home"]);
            }
            _ => panic!("expected simple command"),
        }
    }

    #[test]
    fn parses_pipeline() {
        let node = parse("ps | grep ai");
        assert!(matches!(node, Node::Pipe(_, _)));
    }

    #[test]
    fn parses_assignments() {
        let node = parse("FOO=bar echo hi");
        match node {
            Node::Simple(cmd) => {
                assert_eq!(cmd.assignments, vec![("FOO".into(), "bar".into())]);
                assert_eq!(cmd.name, "echo");
            }
            _ => panic!("expected simple command"),
        }
    }

    #[test]
    fn parses_redirection() {
        let node = parse("echo hi > file.txt");
        assert!(matches!(
            node,
            Node::Redir {
                dir: RedirDir::Out,
                ..
            }
        ));
    }

    #[test]
    fn parses_sequence() {
        let node = parse("echo a; echo b && echo c || echo d");
        assert!(matches!(node, Node::Sequence(_, _)));
    }
}
