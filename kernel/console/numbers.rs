pub fn evaluate(input: &str) -> Result<i64, &'static str> {
    let mut parser = Parser(input.as_bytes());
    let number = parser.expr(0, 0)?;
    if parser.peek().is_some() {
        return Err("unexpected token");
    }
    Ok(number)
}

struct Parser<'a>(&'a [u8]);

impl Parser<'_> {
    fn peek(&mut self) -> Option<u8> {
        while self.0.first() == Some(&b' ') {
            self.0 = &self.0[1..];
        }
        self.0.first().copied()
    }

    fn expr(&mut self, min: u8, depth: u8) -> Result<i64, &'static str> {
        if depth >= 16 {
            return Err("expression nested too deeply");
        }
        let mut left = match self.peek() {
            Some(b'(') => {
                self.0 = &self.0[1..];
                let value = self.expr(0, depth + 1)?;
                if self.peek() != Some(b')') {
                    return Err("expected closing parenthesis");
                }
                self.0 = &self.0[1..];
                value
            }
            Some(b'-') => {
                self.0 = &self.0[1..];
                self.expr(7, depth + 1)?
                    .checked_neg()
                    .ok_or("integer overflow")?
            }
            _ => self.number()?,
        };
        while let Some(op) = self.peek() {
            let priority = match op {
                b'|' => 1,
                b'^' => 2,
                b'&' => 3,
                b'+' | b'-' => 4,
                b'*' | b'/' | b'%' => 5,
                _ => break,
            };
            if priority < min {
                break;
            }
            self.0 = &self.0[1..];
            let right = self.expr(priority + 1, depth + 1)?;
            left = match op {
                b'+' => left.checked_add(right),
                b'-' => left.checked_sub(right),
                b'*' => left.checked_mul(right),
                b'/' if right != 0 => left.checked_div(right),
                b'%' if right != 0 => left.checked_rem(right),
                b'/' | b'%' => return Err("division by zero"),
                b'|' => Some(left | right),
                b'^' => Some(left ^ right),
                b'&' => Some(left & right),
                _ => unreachable!(),
            }
            .ok_or("integer overflow")?;
        }
        Ok(left)
    }

    fn number(&mut self) -> Result<i64, &'static str> {
        let radix = if self.0.starts_with(b"0x") {
            16
        } else if self.0.starts_with(b"0b") {
            2
        } else if self.0.starts_with(b"0o") {
            8
        } else {
            10
        };
        if radix != 10 {
            self.0 = &self.0[2..];
        }
        let mut value = 0i64;
        let mut digits = 0;
        while let Some(digit) = self.0.first().and_then(|b| char::from(*b).to_digit(radix)) {
            self.0 = &self.0[1..];
            value = value
                .checked_mul(i64::from(radix))
                .and_then(|n| n.checked_add(i64::from(digit)))
                .ok_or("integer overflow")?;
            digits += 1;
        }
        if digits == 0 {
            Err("expected an integer")
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate;

    #[test]
    fn precedence_bases_and_errors() {
        assert_eq!(evaluate("0xff & (2 + 3 * 4) | 0b1"), Ok(15));
        assert_eq!(evaluate("100 - 12 - 3 * -2"), Ok(94));
        assert_eq!(evaluate("42 % 5 + 0o10"), Ok(10));
        assert_eq!(evaluate("2 / 0"), Err("division by zero"));
        assert_eq!(evaluate("9223372036854775807 + 1"), Err("integer overflow"));
        assert!(evaluate("0b12").is_err());
        assert!(evaluate("((((((((((((((((((1))))))))))))))))))").is_err());
    }
}
