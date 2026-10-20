//! En liten formeltolk för modellernas formler.
//!
//! Formler kan använda namnen på andra delar i modellen (även namn med
//! mellanslag och å/ä/ö), tal, räknesätten + − * / ^, jämförelser och
//! funktioner som `om(villkor; då; annars)`, `min`, `max` och `abs`.

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(usize),
    Time,
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Call(Func, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Func {
    Min,
    Max,
    Abs,
    If,
    Sqrt,
    Exp,
    Ln,
    Sin,
    Cos,
    Round,
    Floor,
    Ceil,
}

impl Func {
    fn from_name(s: &str) -> Option<Func> {
        Some(match s {
            "min" => Func::Min,
            "max" => Func::Max,
            "abs" | "absolut" => Func::Abs,
            "om" | "if" => Func::If,
            "rot" | "sqrt" => Func::Sqrt,
            "exp" => Func::Exp,
            "ln" => Func::Ln,
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "avrunda" | "round" => Func::Round,
            "golv" | "floor" => Func::Floor,
            "tak" | "ceil" => Func::Ceil,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Func::Min => "min",
            Func::Max => "max",
            Func::Abs => "abs",
            Func::If => "om",
            Func::Sqrt => "rot",
            Func::Exp => "exp",
            Func::Ln => "ln",
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Round => "avrunda",
            Func::Floor => "golv",
            Func::Ceil => "tak",
        }
    }

    /// Minsta och största antal värden funktionen tar.
    fn arity(self) -> (usize, usize) {
        match self {
            Func::Min | Func::Max => (2, usize::MAX),
            Func::If => (3, 3),
            _ => (1, 1),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Var(usize),
    Func(Func),
    Time,
    LParen,
    RParen,
    Sep,
    Op(Op),
    Not,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Gör om ett namn till den form som används vid jämförelser.
pub fn normalize_name(name: &str) -> Vec<char> {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(lower)
        .collect()
}

/// Försöker matcha något av namnen vid position `i`. Ger (längd, index).
fn match_name(text: &[char], i: usize, names: &[(Vec<char>, usize)]) -> Option<(usize, usize)> {
    for (name, idx) in names {
        let end = i + name.len();
        if end <= text.len()
            && text[i..end] == name[..]
            && (end == text.len() || !is_word_char(text[end]))
            && (i == 0 || !is_word_char(text[i - 1]))
        {
            return Some((name.len(), *idx));
        }
    }
    None
}

fn sorted_names(names: &[String]) -> Vec<(Vec<char>, usize)> {
    let mut list: Vec<(Vec<char>, usize)> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (normalize_name(n), i))
        .filter(|(n, _)| !n.is_empty())
        .collect();
    list.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    list
}

/// Text där flera mellanslag i rad har slagits ihop: (gemener, originaltecken).
fn squeeze(src: &str) -> (Vec<char>, Vec<char>) {
    let (mut low, mut orig) = (Vec::new(), Vec::new());
    let mut last_space = false;
    for c in src.chars() {
        if c.is_whitespace() {
            if !last_space {
                low.push(' ');
                orig.push(' ');
            }
            last_space = true;
        } else {
            low.push(lower(c));
            orig.push(c);
            last_space = false;
        }
    }
    (low, orig)
}

fn tokenize(src: &str, names: &[String]) -> Result<Vec<Tok>, String> {
    let (chars, original) = squeeze(src);
    let names = sorted_names(names);
    let mut toks = Vec::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ' ' {
            i += 1;
            continue;
        }
        if is_word_char(c) {
            if let Some((len, idx)) = match_name(&chars, i, &names) {
                toks.push(Tok::Var(idx));
                i += len;
                continue;
            }
        }
        if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let mut s = String::new();
            let mut seen_point = false;
            while i < chars.len() {
                let d = chars[i];
                if d.is_ascii_digit() {
                    s.push(d);
                } else if !seen_point
                    && (d == '.'
                        || (d == ',' && depth == 0 && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit())))
                {
                    // Decimalkomma tillåts utanför parenteser, t.ex. "0,15 * x".
                    seen_point = true;
                    s.push('.');
                } else {
                    break;
                }
                i += 1;
            }
            let v: f64 = s.parse().map_err(|_| format!("Konstigt tal: {s}"))?;
            toks.push(Tok::Num(v));
            continue;
        }
        if is_word_char(c) {
            let start = i;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let next_is_paren = chars.get(j) == Some(&'(');
            let tok = match word.as_str() {
                "och" | "and" => Tok::Op(Op::And),
                "eller" | "or" => Tok::Op(Op::Or),
                "inte" | "not" => Tok::Not,
                "tid" | "time" => Tok::Time,
                "pi" => Tok::Num(std::f64::consts::PI),
                _ => match Func::from_name(&word) {
                    Some(f) if next_is_paren => Tok::Func(f),
                    _ => {
                        let word: String = original[start..i].iter().collect();
                        return Err(format!(
                            "Okänt namn: \"{word}\". Finns det något i modellen som heter så? Kolla stavningen."
                        ));
                    }
                },
            };
            toks.push(tok);
            continue;
        }
        let next = chars.get(i + 1).copied();
        let (tok, len) = match (c, next) {
            ('<', Some('=')) => (Tok::Op(Op::Le), 2),
            ('>', Some('=')) => (Tok::Op(Op::Ge), 2),
            ('<', Some('>')) => (Tok::Op(Op::Ne), 2),
            ('!', Some('=')) => (Tok::Op(Op::Ne), 2),
            ('=', Some('=')) => (Tok::Op(Op::Eq), 2),
            ('&', Some('&')) => (Tok::Op(Op::And), 2),
            ('|', Some('|')) => (Tok::Op(Op::Or), 2),
            ('<', _) => (Tok::Op(Op::Lt), 1),
            ('>', _) => (Tok::Op(Op::Gt), 1),
            ('=', _) => (Tok::Op(Op::Eq), 1),
            ('≤', _) => (Tok::Op(Op::Le), 1),
            ('≥', _) => (Tok::Op(Op::Ge), 1),
            ('≠', _) => (Tok::Op(Op::Ne), 1),
            ('!', _) => (Tok::Not, 1),
            ('+', _) => (Tok::Op(Op::Add), 1),
            ('-' | '−' | '–', _) => (Tok::Op(Op::Sub), 1),
            ('*' | '×' | '·', _) => (Tok::Op(Op::Mul), 1),
            ('/' | '÷', _) => (Tok::Op(Op::Div), 1),
            ('^', _) => (Tok::Op(Op::Pow), 1),
            ('(', _) => {
                depth += 1;
                (Tok::LParen, 1)
            }
            (')', _) => {
                depth -= 1;
                (Tok::RParen, 1)
            }
            (';' | ',', _) => (Tok::Sep, 1),
            _ => return Err(format!("Okänt tecken: '{c}'")),
        };
        toks.push(tok);
        i += len;
    }
    Ok(toks)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn eat_op(&mut self, ops: &[Op]) -> Option<Op> {
        if let Some(Tok::Op(op)) = self.peek() {
            if ops.contains(op) {
                let op = *op;
                self.pos += 1;
                return Some(op);
            }
        }
        None
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut left = self.and()?;
        while let Some(op) = self.eat_op(&[Op::Or]) {
            let right = self.and()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut left = self.cmp()?;
        while let Some(op) = self.eat_op(&[Op::And]) {
            let right = self.cmp()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn cmp(&mut self) -> Result<Expr, String> {
        let left = self.add()?;
        if let Some(op) = self.eat_op(&[Op::Lt, Op::Le, Op::Gt, Op::Ge, Op::Eq, Op::Ne]) {
            let right = self.add()?;
            return Ok(Expr::Bin(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn add(&mut self) -> Result<Expr, String> {
        let mut left = self.mul()?;
        while let Some(op) = self.eat_op(&[Op::Add, Op::Sub]) {
            let right = self.mul()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn mul(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        while let Some(op) = self.eat_op(&[Op::Mul, Op::Div]) {
            let right = self.unary()?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat_op(&[Op::Sub]).is_some() {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat_op(&[Op::Add]).is_some() {
            return self.unary();
        }
        if self.peek() == Some(&Tok::Not) {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        self.pow()
    }

    fn pow(&mut self) -> Result<Expr, String> {
        let base = self.primary()?;
        if self.eat_op(&[Op::Pow]).is_some() {
            let exp = self.unary()?;
            return Ok(Expr::Bin(Op::Pow, Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Tok::Num(v)) => Ok(Expr::Num(v)),
            Some(Tok::Var(i)) => Ok(Expr::Var(i)),
            Some(Tok::Time) => Ok(Expr::Time),
            Some(Tok::LParen) => {
                let e = self.or()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(e),
                    _ => Err("En avslutande parentes ')' saknas.".into()),
                }
            }
            Some(Tok::Func(f)) => {
                self.next(); // '('
                let mut args = Vec::new();
                if self.peek() != Some(&Tok::RParen) {
                    loop {
                        args.push(self.or()?);
                        match self.next() {
                            Some(Tok::Sep) => continue,
                            Some(Tok::RParen) => break,
                            _ => return Err(format!("{}( ... ) saknar en avslutande parentes ')'.", f.name())),
                        }
                    }
                } else {
                    self.next();
                }
                let (lo, hi) = f.arity();
                if args.len() < lo || args.len() > hi {
                    let need = if lo == hi {
                        format!("{lo}")
                    } else {
                        format!("minst {lo}")
                    };
                    let example = if f == Func::If { " Exempel: om(A > 5; 10; 0)" } else { "" };
                    return Err(format!(
                        "{}( ) behöver {need} värden, åtskilda med semikolon (;).{example}",
                        f.name()
                    ));
                }
                Ok(Expr::Call(f, args))
            }
            Some(Tok::RParen) => Err("Det finns en ')' för mycket, eller så saknas något före den.".into()),
            Some(Tok::Sep) => Err("Semikolon/komma används bara mellan värden i en funktion, t.ex. max(A; B).".into()),
            Some(Tok::Op(_)) | Some(Tok::Not) => Err("Två räknetecken i rad – det saknas ett tal eller namn.".into()),
            None => Err("Formeln tar slut för tidigt – det saknas något i slutet.".into()),
        }
    }
}

/// Tolkar en formel. `names` är namnen på modellens delar; `Expr::Var(i)`
/// pekar på `names[i]`.
pub fn parse(src: &str, names: &[String]) -> Result<Expr, String> {
    if src.trim().is_empty() {
        return Err("Formeln är tom – skriv ett tal eller en uträkning.".into());
    }
    let toks = tokenize(src, names)?;
    let mut p = Parser { toks, pos: 0 };
    let e = p.or()?;
    if p.pos < p.toks.len() {
        return Err(match p.toks[p.pos] {
            Tok::RParen => "Det finns en ')' för mycket.".into(),
            _ => "Något saknas mellan två delar – glömde du ett räknetecken (+ − * /)?".into(),
        });
    }
    Ok(e)
}

fn truth(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

impl Expr {
    pub fn eval(&self, vars: &[f64], t: f64) -> f64 {
        match self {
            Expr::Num(v) => *v,
            Expr::Var(i) => vars[*i],
            Expr::Time => t,
            Expr::Neg(e) => -e.eval(vars, t),
            Expr::Not(e) => truth(e.eval(vars, t) == 0.0),
            Expr::Bin(op, a, b) => {
                let x = a.eval(vars, t);
                let y = b.eval(vars, t);
                match op {
                    Op::Add => x + y,
                    Op::Sub => x - y,
                    Op::Mul => x * y,
                    Op::Div => x / y,
                    Op::Pow => x.powf(y),
                    Op::Lt => truth(x < y),
                    Op::Le => truth(x <= y),
                    Op::Gt => truth(x > y),
                    Op::Ge => truth(x >= y),
                    Op::Eq => truth((x - y).abs() < 1e-9),
                    Op::Ne => truth((x - y).abs() >= 1e-9),
                    Op::And => truth(x != 0.0 && y != 0.0),
                    Op::Or => truth(x != 0.0 || y != 0.0),
                }
            }
            Expr::Call(f, args) => {
                let arg = |i: usize| args[i].eval(vars, t);
                match f {
                    Func::If => {
                        if arg(0) != 0.0 {
                            arg(1)
                        } else {
                            arg(2)
                        }
                    }
                    Func::Min => args.iter().map(|a| a.eval(vars, t)).fold(f64::INFINITY, f64::min),
                    Func::Max => args.iter().map(|a| a.eval(vars, t)).fold(f64::NEG_INFINITY, f64::max),
                    Func::Abs => arg(0).abs(),
                    Func::Sqrt => arg(0).sqrt(),
                    Func::Exp => arg(0).exp(),
                    Func::Ln => arg(0).ln(),
                    Func::Sin => arg(0).sin(),
                    Func::Cos => arg(0).cos(),
                    Func::Round => arg(0).round(),
                    Func::Floor => arg(0).floor(),
                    Func::Ceil => arg(0).ceil(),
                }
            }
        }
    }

    /// Samlar index för alla namn som formeln använder.
    pub fn vars(&self, out: &mut Vec<usize>) {
        match self {
            Expr::Var(i) => {
                if !out.contains(i) {
                    out.push(*i)
                }
            }
            Expr::Num(_) | Expr::Time => {}
            Expr::Neg(e) | Expr::Not(e) => e.vars(out),
            Expr::Bin(_, a, b) => {
                a.vars(out);
                b.vars(out);
            }
            Expr::Call(_, args) => args.iter().for_each(|a| a.vars(out)),
        }
    }
}

/// Byter ut ett namn mot ett annat i en formel (hela ord, oavsett skiftläge).
pub fn replace_name(formula: &str, old: &str, new: &str) -> String {
    let old_n = normalize_name(old);
    if old_n.is_empty() {
        return formula.to_string();
    }
    let chars: Vec<char> = formula.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    'outer: while i < chars.len() {
        if is_word_char(chars[i]) && (i == 0 || !is_word_char(chars[i - 1])) {
            // Matcha namnet tecken för tecken, där mellanslag i formeln får vara fler än ett.
            let mut j = i;
            let mut k = 0;
            while k < old_n.len() && j < chars.len() {
                if old_n[k] == ' ' {
                    if !chars[j].is_whitespace() {
                        break;
                    }
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    k += 1;
                } else if lower(chars[j]) == old_n[k] {
                    j += 1;
                    k += 1;
                } else {
                    break;
                }
            }
            if k == old_n.len() && (j == chars.len() || !is_word_char(chars[j])) {
                out.push_str(new.trim());
                i = j;
                continue 'outer;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["Inne temperatur".into(), "Ute".into(), "COP".into(), "Inne".into()]
    }

    fn ev(src: &str, vars: &[f64]) -> f64 {
        parse(src, &names()).unwrap().eval(vars, 2.0)
    }

    #[test]
    fn arithmetic_and_precedence() {
        let v = [0.0; 4];
        assert_eq!(ev("1 + 2 * 3", &v), 7.0);
        assert_eq!(ev("(1 + 2) * 3", &v), 9.0);
        assert_eq!(ev("-2^2", &v), -4.0);
        assert_eq!(ev("2^3^2", &v), 512.0);
        assert_eq!(ev("10 / 4", &v), 2.5);
        assert_eq!(ev("0,5 * 4", &v), 2.0);
        assert_eq!(ev("tid * 3", &v), 6.0);
    }

    #[test]
    fn names_with_spaces_and_case() {
        let v = [21.0, -5.0, 3.0, 1.0];
        assert_eq!(ev("Inne temperatur - Ute", &v), 26.0);
        assert_eq!(ev("inne   TEMPERATUR*cop", &v), 63.0);
        // Det längsta namnet ska vinna, men det kortare går också att använda.
        assert_eq!(ev("Inne + 1", &v), 2.0);
    }

    #[test]
    fn functions() {
        let v = [21.0, -5.0, 3.0, 1.0];
        assert_eq!(ev("om(Ute < 0; 1; 2)", &v), 1.0);
        assert_eq!(ev("if(Ute > 0, 1, 2)", &v), 2.0);
        assert_eq!(ev("max(0; min(5; COP; 4))", &v), 3.0);
        assert_eq!(ev("abs(Ute)", &v), 5.0);
        assert_eq!(ev("Ute < 0 och COP >= 3", &v), 1.0);
        assert_eq!(ev("inte(COP = 3)", &v), 0.0);
    }

    #[test]
    fn errors_are_friendly() {
        assert!(parse("", &names()).unwrap_err().contains("tom"));
        assert!(parse("Banan * 2", &names()).unwrap_err().contains("Banan"));
        assert!(parse("(1 + 2", &names()).unwrap_err().contains(")"));
        assert!(parse("1 +", &names()).is_err());
        assert!(parse("om(1; 2)", &names()).unwrap_err().contains("3"));
        assert!(parse("2 COP", &names()).is_err());
        assert!(parse("1 # 2", &names()).is_err());
    }

    #[test]
    fn collects_vars() {
        let e = parse("Inne temperatur - Ute * Ute", &names()).unwrap();
        let mut v = Vec::new();
        e.vars(&mut v);
        assert_eq!(v, vec![0, 1]);
    }

    #[test]
    fn renames() {
        assert_eq!(replace_name("Ute*2 + uteluft + UTE", "ute", "Utomhus"), "Utomhus*2 + uteluft + Utomhus");
        assert_eq!(replace_name("Lager  1 / 2", "Lager 1", "Vatten"), "Vatten / 2");
    }
}
