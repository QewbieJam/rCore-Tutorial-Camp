// minicc —— 一个能编译"函数调用"的最小编译器
// 流程: 源码 -> 词法(lex) -> 语法(parse, 递归下降) -> AST -> 代码生成(gen) -> RISC-V 汇编
//
// 语言子集:
//   fn name(a, b) { ... }
//   let x = expr;
//   x = expr;                 // 赋值
//   return expr;
//   if (cond) { ... } else { ... }
//   while (cond) { ... }
//   print(expr);              // 运行时函数
//   表达式: + - * /  < > <= >= == !=  ( )  整数  变量  调用
//
// 目标 ABI: RISC-V SysV (lp64)
//   参数 a0-a7, 返回 a0, 非叶子函数保存 ra, sp 16 字节对齐

use std::collections::HashMap;

// ============================== 词法 ==============================

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(i64),
    Fn, Let, Return, If, Else, While,
    LP, RP, LB, RB, Comma, Semi,
    Plus, Minus, Star, Slash,
    Lt, Gt, Le, Ge, EqEq, Ne, Assign,
}

fn lex(src: &str) -> Vec<Tok> {
    let b: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // 注释：// 到行尾
        if c == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // 数字
        if c.is_ascii_digit() {
            let mut n: i64 = 0;
            while i < b.len() && b[i].is_ascii_digit() {
                n = n * 10 + (b[i] as i64 - '0' as i64);
                i += 1;
            }
            out.push(Tok::Num(n));
            continue;
        }
        // 标识符 / 关键字
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }
            let s: String = b[start..i].iter().collect();
            out.push(match s.as_str() {
                "fn" => Tok::Fn,
                "let" => Tok::Let,
                "return" => Tok::Return,
                "if" => Tok::If,
                "else" => Tok::Else,
                "while" => Tok::While,
                _ => Tok::Ident(s),
            });
            continue;
        }
        // 双字符运算符
        let two: String = if i + 1 < b.len() {
            b[i..i + 2].iter().collect()
        } else {
            String::new()
        };
        let t = match two.as_str() {
            "==" => { i += 2; Tok::EqEq }
            "!=" => { i += 2; Tok::Ne }
            "<=" => { i += 2; Tok::Le }
            ">=" => { i += 2; Tok::Ge }
            _ => {
                i += 1;
                match c {
                    '(' => Tok::LP,
                    ')' => Tok::RP,
                    '{' => Tok::LB,
                    '}' => Tok::RB,
                    ',' => Tok::Comma,
                    ';' => Tok::Semi,
                    '+' => Tok::Plus,
                    '-' => Tok::Minus,
                    '*' => Tok::Star,
                    '/' => Tok::Slash,
                    '<' => Tok::Lt,
                    '>' => Tok::Gt,
                    '=' => Tok::Assign,
                    _ => panic!("词法错误: 未知字符 {:?}", c),
                }
            }
        };
        out.push(t);
    }
    out
}

// ============================== AST ==============================

#[derive(Debug, Clone, Copy)]
enum BinOp { Add, Sub, Mul, Div, Lt, Gt, Le, Ge, Eq, Ne }

#[derive(Debug, Clone)]
enum Expr {
    Num(i64),
    Var(String),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    Return(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    ExprStmt(Expr),
}

#[derive(Debug)]
struct Func {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

// ============================== 语法 ==============================

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn is(&self, t: &Tok) -> bool {
        self.peek() == Some(t)
    }
    fn next(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        t
    }
    fn eat(&mut self, t: &Tok) {
        let got = self.next();
        assert!(got == *t, "语法错误: 期望 {:?}, 得到 {:?} (token #{})", t, got, self.pos);
    }

    fn program(&mut self) -> Vec<Func> {
        let mut v = Vec::new();
        while self.peek().is_some() {
            v.push(self.function());
        }
        v
    }

    fn function(&mut self) -> Func {
        self.eat(&Tok::Fn);
        let name = match self.next() {
            Tok::Ident(s) => s,
            t => panic!("函数名错误: {:?}", t),
        };
        self.eat(&Tok::LP);
        let mut params = Vec::new();
        if !self.is(&Tok::RP) {
            loop {
                match self.next() {
                    Tok::Ident(s) => params.push(s),
                    t => panic!("参数名错误: {:?}", t),
                }
                if self.is(&Tok::Comma) {
                    self.next();
                } else {
                    break;
                }
            }
        }
        self.eat(&Tok::RP);
        let body = self.block();
        Func { name, params, body }
    }

    fn block(&mut self) -> Vec<Stmt> {
        self.eat(&Tok::LB);
        let mut v = Vec::new();
        while !self.is(&Tok::RB) {
            v.push(self.stmt());
        }
        self.eat(&Tok::RB);
        v
    }

    fn stmt(&mut self) -> Stmt {
        match self.peek().cloned() {
            Some(Tok::Let) => {
                self.next();
                let name = match self.next() {
                    Tok::Ident(s) => s,
                    t => panic!("let 变量名错误: {:?}", t),
                };
                self.eat(&Tok::Assign);
                let e = self.expr();
                self.eat(&Tok::Semi);
                Stmt::Let(name, e)
            }
            Some(Tok::Return) => {
                self.next();
                if self.is(&Tok::Semi) {
                    self.next();
                    Stmt::Return(Expr::Num(0))
                } else {
                    let e = self.expr();
                    self.eat(&Tok::Semi);
                    Stmt::Return(e)
                }
            }
            Some(Tok::If) => {
                self.next();
                self.eat(&Tok::LP);
                let c = self.expr();
                self.eat(&Tok::RP);
                let then = self.block();
                let els = if self.is(&Tok::Else) {
                    self.next();
                    self.block()
                } else {
                    Vec::new()
                };
                Stmt::If(c, then, els)
            }
            Some(Tok::While) => {
                self.next();
                self.eat(&Tok::LP);
                let c = self.expr();
                self.eat(&Tok::RP);
                let body = self.block();
                Stmt::While(c, body)
            }
            _ => {
                let e = self.expr();
                if self.is(&Tok::Assign) {
                    self.next();
                    let v = self.expr();
                    self.eat(&Tok::Semi);
                    match e {
                        Expr::Var(n) => Stmt::Assign(n, v),
                        _ => panic!("赋值左侧必须是变量"),
                    }
                } else {
                    self.eat(&Tok::Semi);
                    Stmt::ExprStmt(e)
                }
            }
        }
    }

    // 优先级: cmp < add < mul < primary
    fn expr(&mut self) -> Expr {
        self.cmp()
    }
    fn cmp(&mut self) -> Expr {
        let mut l = self.add();
        loop {
            let op = match self.peek() {
                Some(Tok::Lt) => BinOp::Lt,
                Some(Tok::Gt) => BinOp::Gt,
                Some(Tok::Le) => BinOp::Le,
                Some(Tok::Ge) => BinOp::Ge,
                Some(Tok::EqEq) => BinOp::Eq,
                Some(Tok::Ne) => BinOp::Ne,
                _ => break,
            };
            self.next();
            let r = self.add();
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        l
    }
    fn add(&mut self) -> Expr {
        let mut l = self.mul();
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.next();
            let r = self.mul();
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        l
    }
    fn mul(&mut self) -> Expr {
        let mut l = self.primary();
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => BinOp::Mul,
                Some(Tok::Slash) => BinOp::Div,
                _ => break,
            };
            self.next();
            let r = self.primary();
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        l
    }
    fn primary(&mut self) -> Expr {
        match self.next() {
            Tok::Num(n) => Expr::Num(n),
            Tok::Minus => Expr::Bin(BinOp::Sub, Box::new(Expr::Num(0)), Box::new(self.primary())),
            Tok::LP => {
                let e = self.expr();
                self.eat(&Tok::RP);
                e
            }
            Tok::Ident(name) => {
                if self.is(&Tok::LP) {
                    self.next();
                    let mut args = Vec::new();
                    if !self.is(&Tok::RP) {
                        loop {
                            args.push(self.expr());
                            if self.is(&Tok::Comma) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    }
                    self.eat(&Tok::RP);
                    Expr::Call(name, args)
                } else {
                    Expr::Var(name)
                }
            }
            t => panic!("表达式错误: 意外的 token {:?}", t),
        }
    }
}

// ============================== 代码生成 ==============================

struct Gen {
    out: String,
    label: u32,
    slots: HashMap<String, i64>,
    frame: i64,
}

fn align16(n: i64) -> i64 {
    (n + 15) / 16 * 16
}

/// 收集函数体内所有 let 变量名（简单起见：忽略作用域，全部塞进一个帧）
fn collect_locals(stmts: &[Stmt], out: &mut Vec<String>) {
    for s in stmts {
        match s {
            Stmt::Let(n, _) => out.push(n.clone()),
            Stmt::If(_, a, b) => {
                collect_locals(a, out);
                collect_locals(b, out);
            }
            Stmt::While(_, body) => collect_locals(body, out),
            _ => {}
        }
    }
}

impl Gen {
    fn new() -> Self {
        Gen { out: String::new(), label: 0, slots: HashMap::new(), frame: 0 }
    }
    fn emit(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }
    fn label(&mut self) -> String {
        self.label += 1;
        format!(".L{}", self.label)
    }

    fn function(&mut self, f: &Func) {
        // --- 分配栈槽：参数 + 局部变量 ---
        self.slots.clear();
        let mut all: Vec<String> = f.params.clone();
        collect_locals(&f.body, &mut all);
        // 帧布局（一律相对 s0，因为表达式求值时 sp 会临时变动）：
        //   -8(s0)        : ra
        //   -16(s0)       : 旧 s0
        //   -(24+8*i)(s0) : 第 i 个槽
        let n = all.len() as i64;
        self.frame = align16(16 + 8 * n);
        for (i, name) in all.iter().enumerate() {
            self.slots.insert(name.clone(), -(24 + 8 * (i as i64)));
        }

        let frame = self.frame;
        self.emit("");
        self.emit(&format!(".globl {}", f.name));
        self.emit(&format!("{}:", f.name));
        self.emit(&format!("    addi sp, sp, -{}", frame));
        self.emit(&format!("    sd ra, {}(sp)", frame - 8));
        self.emit(&format!("    sd s0, {}(sp)", frame - 16));
        self.emit(&format!("    addi s0, sp, {}", frame));

        // 参数从 a0-a7 落到栈槽
        for (i, p) in f.params.iter().enumerate() {
            let off = self.slots[p];
            self.emit(&format!("    sd a{}, {}(s0)", i, off));
        }

        let ret_label = format!(".Lret_{}", f.name);

        for s in &f.body {
            self.stmt(s, &ret_label);
        }

        // 函数末尾默认返回 0
        self.emit("    li a0, 0");
        self.emit(&format!("{}:", ret_label));
        self.emit(&format!("    ld ra, {}(sp)", frame - 8));
        self.emit(&format!("    ld s0, {}(sp)", frame - 16));
        self.emit(&format!("    addi sp, sp, {}", frame));
        self.emit("    ret");
    }

    fn stmt(&mut self, s: &Stmt, ret_label: &str) {
        match s {
            Stmt::Let(name, e) | Stmt::Assign(name, e) => {
                self.expr(e);
                let off = self.slots[name];
                self.emit(&format!("    sd a0, {}(s0)", off));
            }
            Stmt::ExprStmt(e) => {
                self.expr(e);
            }
            Stmt::Return(e) => {
                self.expr(e);
                self.emit(&format!("    j {}", ret_label));
            }
            Stmt::If(c, then, els) => {
                let l_else = self.label();
                let l_end = self.label();
                self.expr(c);
                self.emit(&format!("    beqz a0, {}", l_else));
                for st in then {
                    self.stmt(st, ret_label);
                }
                self.emit(&format!("    j {}", l_end));
                self.emit(&format!("{}:", l_else));
                for st in els {
                    self.stmt(st, ret_label);
                }
                self.emit(&format!("{}:", l_end));
            }
            Stmt::While(c, body) => {
                let l_begin = self.label();
                let l_end = self.label();
                self.emit(&format!("{}:", l_begin));
                self.expr(c);
                self.emit(&format!("    beqz a0, {}", l_end));
                for st in body {
                    self.stmt(st, ret_label);
                }
                self.emit(&format!("    j {}", l_begin));
                self.emit(&format!("{}:", l_end));
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Num(n) => self.emit(&format!("    li a0, {}", n)),
            Expr::Var(name) => {
                let off = self.slots[name];
                self.emit(&format!("    ld a0, {}(s0)", off));
            }
            Expr::Bin(op, l, r) => {
                // 规则：表达式结果放 a0；二元运算"左压栈、算右、弹回 a1"
                self.expr(l);
                self.emit("    addi sp, sp, -16");
                self.emit("    sd a0, 0(sp)");
                self.expr(r);
                self.emit("    mv a1, a0");
                self.emit("    ld a0, 0(sp)");
                self.emit("    addi sp, sp, 16");
                match op {
                    BinOp::Add => self.emit("    add a0, a0, a1"),
                    BinOp::Sub => self.emit("    sub a0, a0, a1"),
                    BinOp::Mul => self.emit("    mul a0, a0, a1"),
                    BinOp::Div => self.emit("    div a0, a0, a1"),
                    BinOp::Lt => self.emit("    slt a0, a0, a1"),
                    BinOp::Gt => self.emit("    slt a0, a1, a0"),
                    BinOp::Le => {
                        self.emit("    slt a0, a1, a0");
                        self.emit("    xori a0, a0, 1");
                    }
                    BinOp::Ge => {
                        self.emit("    slt a0, a0, a1");
                        self.emit("    xori a0, a0, 1");
                    }
                    BinOp::Eq => {
                        self.emit("    xor a0, a0, a1");
                        self.emit("    seqz a0, a0");
                    }
                    BinOp::Ne => {
                        self.emit("    xor a0, a0, a1");
                        self.emit("    snez a0, a0");
                    }
                }
            }
            Expr::Call(name, args) => {
                // 关键：参数先全部压栈，再逆序弹进 a0-a7
                // 否则算后面参数时，前面的参数寄存器已被嵌套调用破坏
                let n = args.len();
                for a in args {
                    self.expr(a);
                    self.emit("    addi sp, sp, -16");
                    self.emit("    sd a0, 0(sp)");
                }
                for i in 0..n {
                    let off = 16 * (n - 1 - i);
                    self.emit(&format!("    ld a{}, {}(sp)", i, off));
                }
                if n > 0 {
                    self.emit(&format!("    addi sp, sp, {}", 16 * n));
                }
                self.emit(&format!("    call {}", name));
            }
        }
    }
}

// ============================== 主程序 ==============================

fn main() {
    let path = std::env::args().nth(1).expect("用法: minicc <file.mc>");
    let src = std::fs::read_to_string(&path).expect("读不到源文件");

    let toks = lex(&src);
    let mut p = Parser { toks, pos: 0 };
    let funcs = p.program();

    let mut g = Gen::new();
    g.emit("    .text");
    for f in &funcs {
        g.function(f);
    }
    print!("{}", g.out);
}
