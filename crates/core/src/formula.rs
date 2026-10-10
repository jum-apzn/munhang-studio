//! Conservative LaTeX-to-Hancom conversion for the local formula OCR boundary.
//! This is not a complete TeX parser. Unknown constructs yield no script.
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FormulaConversion {
    pub latex: String,
    pub script: String,
    pub warnings: Vec<String>,
}

pub fn convert_latex(latex: &str) -> FormulaConversion {
    let clean = latex.trim().trim_matches('$').trim();
    let mut result = FormulaConversion {
        latex: latex.to_owned(),
        script: String::new(),
        warnings: Vec::new(),
    };
    if clean.len() > 16_384 {
        result
            .warnings
            .push("수식이 16 KB 제한을 초과했습니다.".into());
        return result;
    }
    let mut parser = Parser {
        chars: clean.chars().collect(),
        at: 0,
        depth: 0,
    };
    match parser.sequence(false) {
        Ok(script) if !script.trim().is_empty() => result.script = script.trim().to_owned(),
        Ok(_) => result.warnings.push("인식된 수식이 비어 있습니다.".into()),
        Err(error) => result.warnings.push(error),
    }
    result
}
struct Parser {
    chars: Vec<char>,
    at: usize,
    depth: usize,
}
impl Parser {
    fn spaces(&mut self) {
        while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
            self.at += 1;
        }
    }
    fn argument(&mut self) -> Result<String, String> {
        self.spaces();
        if self.chars.get(self.at) == Some(&'{') {
            self.at += 1;
            self.sequence(true)
        } else {
            self.atom()
        }
    }
    fn sequence(&mut self, closing: bool) -> Result<String, String> {
        self.depth += 1;
        if self.depth > 64 {
            return Err("수식의 중첩이 너무 깊습니다.".into());
        }
        let mut output = String::new();
        while let Some(c) = self.chars.get(self.at) {
            if *c == '}' {
                if !closing {
                    return Err("닫는 중괄호가 짝이 맞지 않습니다.".into());
                }
                self.at += 1;
                self.depth -= 1;
                return Ok(output);
            }
            output.push_str(&self.atom()?);
        }
        self.depth -= 1;
        if closing {
            Err("수식의 중괄호가 닫히지 않았습니다.".into())
        } else {
            Ok(output)
        }
    }
    fn atom(&mut self) -> Result<String, String> {
        // Every recursive parse edge eventually enters atom(), including bare
        // superscripts/subscripts and commands such as \sqrt without braces.
        // A sequence-only limit does not cover those paths.
        self.depth += 1;
        if self.depth > 64 {
            self.depth -= 1;
            return Err("수식의 중첩이 너무 깊습니다.".into());
        }
        let result = self.atom_inner();
        self.depth -= 1;
        result
    }
    fn atom_inner(&mut self) -> Result<String, String> {
        let c = *self.chars.get(self.at).ok_or("수식 인수가 빠졌습니다.")?;
        self.at += 1;
        match c {
            '{' => Ok(format!("{{ {} }}", self.sequence(true)?)),
            '}' => Err("수식 인수가 빠졌습니다.".into()),
            '\\' => self.command(),
            '^' | '_' => Ok(format!("{c}{{ {} }}", self.argument()?)),
            '&' | '#' | '%' | '~' => Err(format!(
                "지원하지 않는 LaTeX 기호 '{c}'입니다. 원문을 확인하고 직접 수정하세요."
            )),
            c if c.is_control() && !c.is_whitespace() => Err("수식에 제어 문자가 있습니다.".into()),
            c => Ok(c.to_string()),
        }
    }
    fn command(&mut self) -> Result<String, String> {
        let start = self.at;
        while self
            .chars
            .get(self.at)
            .is_some_and(|c| c.is_ascii_alphabetic())
        {
            self.at += 1;
        }
        let name: String = self.chars[start..self.at].iter().collect();
        match name.as_str() {
            "frac"|"dfrac"|"tfrac" => {let a=self.argument()?;let b=self.argument()?;Ok(format!("{{ {{ {a} }} over {{ {b} }} }}"))},
            "sqrt" => {self.spaces();if self.chars.get(self.at)==Some(&'['){return Err("첨자가 있는 근호는 직접 수정해 주세요.".into());}Ok(format!("sqrt {{ {} }}",self.argument()?))},
            "left"|"right"|"displaystyle"|"textstyle" => Ok(String::new()),
            "cdot" => Ok(" cdot ".into()), "times"=>Ok(" times ".into()), "div"=>Ok(" div ".into()),
            "le"|"leq"=>Ok(" <= ".into()), "ge"|"geq"=>Ok(" >= ".into()), "ne"|"neq"=>Ok(" != ".into()),
            "pm"=>Ok(" +- ".into()), "mp"=>Ok(" -+ ".into()), "infty"=>Ok(" inf ".into()),
            "alpha"|"beta"|"gamma"|"delta"|"epsilon"|"theta"|"lambda"|"mu"|"pi"|"rho"|"sigma"|"phi"|"omega"|
            "Gamma"|"Delta"|"Theta"|"Lambda"|"Pi"|"Sigma"|"Phi"|"Omega"|
            "sin"|"cos"|"tan"|"log"|"ln"|"lim"|"sum"|"prod"|"int"|"in"|"notin"|"approx"|"equiv" => Ok(format!(" {name} ")),
            "mathrm"=>Ok(format!("rm {{ {} }}",self.argument()?)),
            "quad"|"qquad"=>Ok(" `` ".into()),
            "" => {let next=*self.chars.get(self.at).ok_or("수식 명령이 완성되지 않았습니다.")?;self.at+=1;match next{','|';'|' '|':'|'!'=>Ok(" ".into()), '('|')'|'['|']'=>Ok(String::new()), _=>Err(format!("지원하지 않는 LaTeX 이스케이프 \\{next}입니다."))}},
            _ => Err(format!("LaTeX 명령 \\{name}은 아직 변환하지 못합니다. 원문을 확인하고 한컴 수식으로 직접 수정하세요.")),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_formula() {
        assert_eq!(
            convert_latex("x^2 + y_1 = 4").script,
            "x^{ 2 } + y_{ 1 } = 4"
        );
    }
    #[test]
    fn fraction_and_root() {
        let c = convert_latex(r"\frac{1}{\sqrt{x+1}}");
        assert!(c.warnings.is_empty());
        assert!(c.script.contains("over"));
        assert!(c.script.contains("sqrt"));
    }
    #[test]
    fn rejects_unknown() {
        let c = convert_latex(r"\begin{matrix}1&2\end{matrix}");
        assert!(c.script.is_empty());
        assert!(!c.warnings.is_empty());
    }
    #[test]
    fn rejects_malformed() {
        for s in ["{x", "x}", "x^", r"\frac{1}", r"\sqrt[3]{x}"] {
            assert!(convert_latex(s).script.is_empty(), "{s}");
        }
    }
    #[test]
    fn removes_math_delimiters() {
        assert_eq!(convert_latex("$$x+1$$").script, "x+1");
        assert_eq!(convert_latex(r"\(x+1\)").script, "x+1");
    }
    #[test]
    fn preserves_raw() {
        assert_eq!(convert_latex(r"\unsupported{x}").latex, r"\unsupported{x}");
    }
    #[test]
    fn bounds_depth() {
        assert!(
            !convert_latex(&format!("{}x{}", "{".repeat(80), "}".repeat(80)))
                .warnings
                .is_empty()
        );
    }
    #[test]
    fn never_claims_empty_valid() {
        assert!(!convert_latex(" ").warnings.is_empty());
    }

    #[test]
    fn bounds_unbraced_superscript_and_subscript_recursion() {
        for marker in ['^', '_'] {
            let input = format!("{}x", marker.to_string().repeat(16_000));
            assert!(input.len() <= 16_384);
            let result = convert_latex(&input);
            assert!(result.script.is_empty());
            assert!(result.warnings.iter().any(|w| w.contains("중첩")));
        }
    }
    #[test]
    fn bounds_unbraced_command_recursion() {
        for command in [r"\sqrt ", r"\frac "] {
            let input = format!("{}x", command.repeat(1_000));
            let result = convert_latex(&input);
            assert!(result.script.is_empty());
            assert!(result.warnings.iter().any(|w| w.contains("중첩")));
        }
    }
}
