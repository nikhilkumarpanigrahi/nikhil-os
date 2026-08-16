//! `nish` shell — lexer.
//!
//! Input → tokens. Supports words, quoting (single/double/escapes), pipes,
//! redirection, and command separators.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Pipe,
    And,
    Or,
    Semicolon,
    RedirIn,
    RedirOut,
    RedirAppend,
    Eof,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' => i += 1,
            '|' => {
                tokens.push(Token::Pipe);
                i += 1;
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::RedirAppend);
                    i += 2;
                } else {
                    tokens.push(Token::RedirOut);
                    i += 1;
                }
            }
            '<' => {
                tokens.push(Token::RedirIn);
                i += 1;
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::And);
                    i += 2;
                } else {
                    // Treat a lone `&` as background marker; unsupported here.
                    tokens.push(Token::Word("&".to_string()));
                    i += 1;
                }
            }
            ';' => {
                tokens.push(Token::Semicolon);
                i += 1;
            }
            '"' => {
                let (word, next) = lex_double_quoted(&chars, i);
                tokens.push(Token::Word(word));
                i = next;
            }
            '\'' => {
                let (word, next) = lex_single_quoted(&chars, i);
                tokens.push(Token::Word(word));
                i = next;
            }
            '\\' => {
                let (word, next) = lex_escaped(&chars, i);
                tokens.push(Token::Word(word));
                i = next;
            }
            _ => {
                let (word, next) = lex_unquoted(&chars, i);
                tokens.push(Token::Word(word));
                i = next;
            }
        }
    }

    tokens.push(Token::Eof);
    tokens
}

fn lex_unquoted(chars: &[char], start: usize) -> (String, usize) {
    let mut word = String::new();
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '|' | '>' | '<' | '&' | ';' => break,
            '"' => {
                let (part, next) = lex_double_quoted(chars, i);
                word.push_str(&part);
                i = next;
            }
            '\'' => {
                let (part, next) = lex_single_quoted(chars, i);
                word.push_str(&part);
                i = next;
            }
            '\\' if i + 1 < chars.len() => {
                word.push(chars[i + 1]);
                i += 2;
            }
            _ => {
                word.push(chars[i]);
                i += 1;
            }
        }
    }
    (word, i)
}

fn lex_double_quoted(chars: &[char], start: usize) -> (String, usize) {
    let mut word = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '"' => return (word, i + 1),
            '\\' if i + 1 < chars.len() => {
                word.push(chars[i + 1]);
                i += 2;
            }
            c => {
                word.push(c);
                i += 1;
            }
        }
    }
    (word, i)
}

fn lex_single_quoted(chars: &[char], start: usize) -> (String, usize) {
    let mut word = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\'' => return (word, i + 1),
            c => {
                word.push(c);
                i += 1;
            }
        }
    }
    (word, i)
}

fn lex_escaped(chars: &[char], start: usize) -> (String, usize) {
    if start + 1 < chars.len() {
        (chars[start + 1].to_string(), start + 2)
    } else {
        (String::new(), start + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_simple_command() {
        let tokens = lex("ls -la /home");
        assert_eq!(
            tokens,
            vec![
                Token::Word("ls".into()),
                Token::Word("-la".into()),
                Token::Word("/home".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn lexes_pipeline() {
        let tokens = lex("ps | grep ai");
        assert_eq!(
            tokens,
            vec![
                Token::Word("ps".into()),
                Token::Pipe,
                Token::Word("grep".into()),
                Token::Word("ai".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn lexes_redirection_and_separators() {
        let tokens = lex("echo hi > file.txt; echo bye >> file.txt");
        assert!(tokens.contains(&Token::RedirOut));
        assert!(tokens.contains(&Token::RedirAppend));
        assert!(tokens.contains(&Token::Semicolon));
    }

    #[test]
    fn lexes_quotes() {
        let tokens = lex("echo 'hello world' \"double $x\"");
        assert_eq!(tokens[1], Token::Word("hello world".into()));
        assert_eq!(tokens[2], Token::Word("double $x".into()));
    }
}
