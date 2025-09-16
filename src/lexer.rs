use std::fmt;

#[derive(Debug, Clone,Copy, PartialEq, Eq)]
pub struct Location {
    line: usize,
    col: usize
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Select,
    From,
    As,
    Table,
    Create,
    Insert,
    Into,
    Values,
    Int,
    Text,
}

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::Select => "select",
            Keyword::From => "from",
            Keyword::As => "as",
            Keyword::Table => "table",
            Keyword::Create => "create",
            Keyword::Insert => "insert",
            Keyword::Into => "into",
            Keyword::Values => "values",
            Keyword::Int => "int",
            Keyword::Text => "text",
        }
    }
}

pub enum Symbol {
    Semicolon,
    Asterix,
    Comma,
    LeftParen,
    RightParen
}

impl Symbol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Symbol::Semicolon => ";",
            Symbol::Asterix => "*",
            Symbol::Comma => ",",
            Symbol::LeftParen => "(",
            Symbol::RightParen => ")",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Symbol,
    Identifier,
    StringLiteral,
    NumericLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    value: String,
    kind: TokenKind,
    loc: Location,
}

impl Token {
    pub fn equals(&self, other: &Token) -> bool {
        self.value == other.value && self.kind == other.kind
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token(value=\"{}\", kind={:?}, loc=({}, {}))",
            self.value, self.kind, self.loc.line, self.loc.col
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pointer: u32,
    loc: Location,
}


pub type LexerFn = fn(&str, Cursor) -> Option<(Token, Cursor)>;

pub fn lex_numeric(source: &str, ic: Cursor) -> Option<(Token, Cursor)> {

    let mut cur = ic; // mutable copy of our input cursor, so that we can move it forward as we are reading characters

    /* 
        Keeping track wether we have seen a period ',' or an exponent marker 'e' || 'E'
        We need to keep track of this so that we can follow the logic of what is considered a valid numerical value
        According to the PostgreSQL documentation (https://www.postgresql.org/docs/current/sql-syntax-lexical.html)
     */
    let mut period_found = false;
    let mut exp_marker_found = false;

    // Iterate over characters starting at current pointer
    while (cur.pointer as usize) < source.len() {
        // SAFETY: assume ASCII
        /*
            start here 
            look at first digit 
            decide what it is (digit, period, exponent)
            t
         */
        let c = source.as_bytes()[cur.pointer as usize] as char;
        cur.loc.col += 1;

        let is_digit = c >= '0' && c <= '9';
        let is_period = c == '.';
        let is_exp_marker = c == 'e' || c == 'E';

        // Rule #1 
        // Must start with digit or period
        if cur.pointer == ic.pointer {
            if !is_digit && !is_period {
                return None;
            }
            period_found = is_period;
            cur.pointer += 1;
            continue;
        }

        if is_period {
            if period_found {
                return None;
            }
            period_found = true;
            cur.pointer += 1;
            continue;
        }

        if is_exp_marker {
            if exp_marker_found {
                return None;
            }
            period_found = true;     // no periods allowed after exp
            exp_marker_found = true;

            // expMarker must be followed by digits
            if (cur.pointer as usize) == source.len() - 1 {
                return None;
            }

            let c_next = source.as_bytes()[cur.pointer as usize + 1] as char;
            cur.pointer += 1;
            cur.loc.col += 1;

            if c_next == '-' || c_next == '+' {
                cur.pointer += 1;
                cur.loc.col += 1;
            }
            continue;
        }

        if !is_digit {
            break;
        }

        cur.pointer += 1;
    }

    // No characters accumulated
    if cur.pointer == ic.pointer {
        return None;
    }

    let value = &source[ic.pointer as usize..cur.pointer as usize];
    Some((
        Token {
            value: value.to_string(),
            kind: TokenKind::NumericLiteral,
            loc: ic.loc,
        },
        cur,
    ))
}



/* 

fn lex_keyword(input: &str, cursor: Cursor) -> Option<(Token, Cursor)> {

}

fn lex_symbol(input: &str, cursor: Cursor) -> Option<(Token, Cursor)> { 

}

fn lex_string(input: &str, cursor: Cursor) -> Option<(Token, Cursor)> { 

}


fn lex_identifier(input: &str, cursor: Cursor) -> Option<(Token, Cursor)> { 

}
*/

pub fn lex(source: String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut cur = Cursor {
        pointer: 0,
        loc: Location {line:1, col:1,}
    };
    'lex: while (cur.pointer as usize) < source.len() {
        let lexers: Vec<LexerFn> = vec![
            //lex_keyword,
            //lex_symbol,
            //lex_string,
            lex_numeric,
            //lex_identifier,
        ];

        for l in lexers {
            if let Some((token, new_cursor)) = l(&source,cur) {
                cur = new_cursor;
    
                if !token.value.is_empty() {
                    tokens.push(token);
                }
    
                continue 'lex;
            }
        }
    
    // Error if no lexer matched
        let hint = if let Some(last) = tokens.last() {
            format!(" after {}", last.value)
        } else {
            "".to_string()
        };

        return Err(format!(
            "Unable to lex token{} at {}:{}",
            hint, cur.loc.line, cur.loc.col
        ));
    }
        Ok(tokens)
}
    

    


#[cfg(test)]
mod tests {
    use super::*;

    fn make_cursor() -> Cursor {
        Cursor {
            pointer: 0,
            loc: Location { line: 1, col: 1 },
        }
    }

    #[test]
    fn test_integer() {
        let source = "123";
        let result = lex_numeric(source, make_cursor());
        assert!(result.is_some(), "Expected to lex an integer");
        let (token, cur) = result.unwrap();
        println!("{:?}", token);
        println!("{:?}", cur);
        assert_eq!(token.value, "123");
        assert_eq!(token.kind, TokenKind::NumericLiteral);
        assert_eq!(cur.pointer as usize, source.len());
    }
    #[test]
    fn test_float() {
        let source = "3.14";
        let result = lex_numeric(source, make_cursor());
        assert!(result.is_some(), "Expected to lex a float");
        let (token, cur) = result.unwrap();
        println!("{:?}", token);
        println!("{:?}", cur);
        assert_eq!(token.value, "3.14");
        assert_eq!(token.kind, TokenKind::NumericLiteral);
        assert_eq!(cur.pointer as usize, source.len());
    }

}