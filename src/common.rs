use Value::{Number, Nil, Boolean};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => write!(f, "nil"),
            Number(n) => write!(f, "{n}"),
            Boolean(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "\"{s}\""),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Nil, Nil) => true,
            (Number(a), Number(b)) => a == b,
            (Boolean(a), Boolean(b)) => *a && *b,
            (Value::String(a), Value::String(b)) => a.eq(b),
            _ => false, 
        }
    }
}

impl Value {
    fn stringfy(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Boolean(b) => b.to_string(),
            Nil => "nil".to_string(),
            Number(n) => n.to_string(),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Nil => false,
            Boolean(b) => *b,
            _ => true,
        }
    }

    pub fn not(self) -> Result<Value, ()> {
        Ok(Boolean(!self.is_truthy()))
    }

    pub fn pos(self) -> Result<Value, ()> {
        match self {
            Number(n) => Ok(Number(n)),
            _ => Err(()),
        }
    }

    pub fn neg(self) -> Result<Value, ()> {
        match self {
            Number(n) => Ok(Number(-n)),
            _ => Err(()),
        }
    }


    pub fn add(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.clone() + b)),
            (Value::String(a), b) => Ok(Value::String(a + &b.stringfy())),
            (a, Value::String(b)) => Ok(Value::String(b.clone() + &a.stringfy())),
            _ => Err(()),
        }
    }

    pub fn sub(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Number(a - b)),
            _ => Err(()),
        }
    }

    pub fn mul(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Number(a * b)),
            _ => Err(()),
        }
    }

    pub fn div(self, other: &Self) -> Result<Value, bool> {
        match (self, other) {
            (_, Number(0.)) => Err(true),
            (Number(a), Number(b)) => Ok(Number(a / b)),
            _ => Err(false),
        }
    }

    pub fn gt(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Boolean(a > *b)),
            _ => Err(())
        }
    }

    pub fn gteq(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Boolean(a >= *b)),
            _ => Err(())
        }
    }

    pub fn ls(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Boolean(a < *b)),
            _ => Err(())
        }
    }

    pub fn lseq(self, other: &Self) -> Result<Value, ()> {
        match (self, other) {
            (Number(a), Number(b)) => Ok(Boolean(a <= *b)),
            _ => Err(())
        }
    }
}