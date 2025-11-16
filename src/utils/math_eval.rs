/// Simple math expression evaluator for basic arithmetic
/// Supports +, -, *, / operations and follows order of operations

pub fn eval_expression(expr: &str) -> Result<usize, String> {
    let expr = expr.trim().replace(" ", "");
    
    if expr.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    // Try direct number parse first
    if let Ok(num) = expr.parse::<usize>() {
        return Ok(num);
    }
    
    // Parse and evaluate the expression
    let tokens = tokenize(&expr)?;
    let result = evaluate(&tokens)?;
    
    Ok(result)
}

#[derive(Debug, Clone)]
enum Token {
    Number(usize),
    Plus,
    Minus,
    Multiply,
    Divide,
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut current_number = String::new();
    
    for ch in expr.chars() {
        match ch {
            '0'..='9' => {
                current_number.push(ch);
            }
            '+' | '-' | '*' | '/' => {
                if !current_number.is_empty() {
                    let num = current_number.parse::<usize>()
                        .map_err(|_| format!("Invalid number: {}", current_number))?;
                    tokens.push(Token::Number(num));
                    current_number.clear();
                } else if tokens.is_empty() || matches!(tokens.last(), Some(Token::Plus | Token::Minus | Token::Multiply | Token::Divide)) {
                    return Err(format!("Invalid operator position: {}", ch));
                }
                
                tokens.push(match ch {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    _ => unreachable!(),
                });
            }
            _ => {
                return Err(format!("Invalid character: {}", ch));
            }
        }
    }
    
    if !current_number.is_empty() {
        let num = current_number.parse::<usize>()
            .map_err(|_| format!("Invalid number: {}", current_number))?;
        tokens.push(Token::Number(num));
    }
    
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    Ok(tokens)
}

fn evaluate(tokens: &[Token]) -> Result<usize, String> {
    if tokens.is_empty() {
        return Err("No tokens to evaluate".to_string());
    }
    
    // First pass: handle * and / (collect results into new token list)
    let mut simplified = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        if let Token::Number(n) = &tokens[i] {
            let mut current_value = *n;
            let mut j = i + 1;
            
            // Keep processing * and / operations
            while j + 1 < tokens.len() {
                match &tokens[j] {
                    Token::Multiply => {
                        if let Token::Number(m) = &tokens[j + 1] {
                            current_value *= m;
                            j += 2;
                        } else {
                            break;
                        }
                    }
                    Token::Divide => {
                        if let Token::Number(m) = &tokens[j + 1] {
                            if *m == 0 {
                                return Err("Division by zero".to_string());
                            }
                            current_value /= m;
                            j += 2;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            
            simplified.push(Token::Number(current_value));
            
            // If we consumed more tokens, check if there's a + or - after
            if j < tokens.len() {
                match &tokens[j] {
                    Token::Plus | Token::Minus => {
                        simplified.push(tokens[j].clone());
                        i = j + 1;
                    }
                    _ => {
                        i = j;
                    }
                }
            } else {
                i = j;
            }
        } else {
            return Err("Expected number in expression".to_string());
        }
    }
    
    // Second pass: handle + and -
    if simplified.is_empty() {
        return Err("No values after multiplication/division".to_string());
    }
    
    let Token::Number(mut result) = simplified[0] else {
        return Err("Expression must start with a number".to_string());
    };
    
    i = 1;
    while i < simplified.len() {
        if i + 1 >= simplified.len() {
            return Err("Invalid expression structure".to_string());
        }
        
        let operator = &simplified[i];
        let Token::Number(operand) = simplified[i + 1] else {
            return Err("Expected number after operator".to_string());
        };
        
        match operator {
            Token::Plus => result += operand,
            Token::Minus => {
                if result < operand {
                    return Err("Result would be negative (usize cannot be negative)".to_string());
                }
                result -= operand;
            }
            _ => return Err("Unexpected operator in simplified expression".to_string()),
        }
        
        i += 2;
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        assert_eq!(eval_expression("42").unwrap(), 42);
        assert_eq!(eval_expression("0").unwrap(), 0);
        assert_eq!(eval_expression("1000").unwrap(), 1000);
    }

    #[test]
    fn test_addition() {
        assert_eq!(eval_expression("2+3").unwrap(), 5);
        assert_eq!(eval_expression("10+20+30").unwrap(), 60);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(eval_expression("10-3").unwrap(), 7);
        assert_eq!(eval_expression("100-50-25").unwrap(), 25);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(eval_expression("8*8").unwrap(), 64);
        assert_eq!(eval_expression("3*4*2").unwrap(), 24);
    }

    #[test]
    fn test_division() {
        assert_eq!(eval_expression("64/8").unwrap(), 8);
        assert_eq!(eval_expression("100/5/2").unwrap(), 10);
    }

    #[test]
    fn test_order_of_operations() {
        assert_eq!(eval_expression("2+3*4").unwrap(), 14);
        assert_eq!(eval_expression("10-6/2").unwrap(), 7);
        assert_eq!(eval_expression("2*3+4*5").unwrap(), 26);
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(eval_expression("100+20*3-10/2").unwrap(), 155);
        assert_eq!(eval_expression("8*8+16").unwrap(), 80);
    }

    #[test]
    fn test_with_spaces() {
        assert_eq!(eval_expression("8 * 8").unwrap(), 64);
        assert_eq!(eval_expression("10 + 20 - 5").unwrap(), 25);
    }

    #[test]
    fn test_division_by_zero() {
        assert!(eval_expression("10/0").is_err());
    }

    #[test]
    fn test_negative_result() {
        assert!(eval_expression("5-10").is_err());
    }

    #[test]
    fn test_invalid_expression() {
        assert!(eval_expression("++5").is_err());
        assert!(eval_expression("5+").is_err());
        assert!(eval_expression("*5").is_err());
        assert!(eval_expression("abc").is_err());
    }
}
