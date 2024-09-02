use std::collections::HashMap;
use std::fmt;

// Enum to represent the different types in the type system
#[derive(Debug, PartialEq, Clone)]
enum Type {
    Int, // Integer type
    Float, // Floating-point type
    String, // String type
    Bool, // Boolean type
    Function(Box<Type>, Box<Type>), // Function type: (input_type, return_type)
}

// Represents a variable with a name and a type
#[derive(Debug, Clone)]
struct Variable {
    name: String,
    var_type: Type,
}

// Represents an expression in our language
#[derive(Debug, Clone)]
enum Expression {
    Variable(String), // Reference to a variable
    Literal(Type), // A literal value with a type
    BinaryOp(Box<Expression>, String, Box<Expression>), // Binary operation with left operand, operator, and right operand
    FunctionCall(Box<Expression>, Vec<Expression>), // Function call with function expression and arguments
}

// Represents a function with a name, parameters, and a return type
#[derive(Debug, Clone)]
struct Function {
    name: String,
    params: HashMap<String, Type>, // Parameters with their types
    return_type: Type, // Return type of the function
}

// Represents the state of the type checker
#[derive(Debug)]
struct TypeChecker {
    variables: HashMap<String, Type>, // Map of variable names to their types
    functions: HashMap<String, Function>, // Map of function names to their definitions
}

// Error types for the type checker
#[derive(Debug)]
enum TypeError {
    UndefinedVariable(String), // Error for using an undefined variable
    TypeMismatch { expected: Type, found: Type }, // Error for type mismatch
    UndefinedFunction(String), // Error for calling an undefined function
    ArgumentMismatch { function: String, expected: Vec<Type>, found: Vec<Type> }, // Error for argument type mismatch
}

// Implementing Display for TypeError to provide human-readable error messages
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::UndefinedVariable(var) => write!(f, "Undefined variable: {}", var),
            TypeError::TypeMismatch { expected, found } => write!(f, "Type mismatch: expected {:?}, found {:?}", expected, found),
            TypeError::UndefinedFunction(func) => write!(f, "Undefined function: {}", func),
            TypeError::ArgumentMismatch { function, expected, found } => write!(f, "Argument mismatch for function '{}': expected {:?}, found {:?}", function, expected, found),
        }
    }
}

impl TypeChecker {
    // Creates a new TypeChecker with empty variable and function maps
    fn new() -> Self {
        TypeChecker {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    // Checks an expression and infers its type
    fn check_expression(&self, expr: &Expression) -> Result<Type, TypeError> {
        match expr {
            // Case for variable expressions
            Expression::Variable(name) => self.variables.get(name)
                .cloned()
                .ok_or(TypeError::UndefinedVariable(name.clone())),

            // Case for literal expressions
            Expression::Literal(lit_type) => Ok(lit_type.clone()),

            // Case for binary operations
            Expression::BinaryOp(left, op, right) => {
                // Recursively check the left and right operands
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                // Handle different operators
                match op.as_str() {
                    "+" | "-" | "*" | "/" => {
                        // Arithmetic operators require both operands to be of the same type
                        if left_type != right_type {
                            Err(TypeError::TypeMismatch { expected: left_type.clone(), found: right_type })
                        } else {
                            Ok(left_type) // Result type is the same as operands' type
                        }
                    },
                    "==" | "!=" => {
                        // Equality operators require both operands to be of the same type
                        if left_type != right_type {
                            Err(TypeError::TypeMismatch { expected: left_type.clone(), found: right_type })
                        } else {
                            Ok(Type::Bool) // Equality results in a boolean
                        }
                    },
                    _ => Err(TypeError::TypeMismatch { expected: left_type, found: right_type }),
                }
            },

            // Case for function calls
            Expression::FunctionCall(func_expr, args) => {
                // Check the type of the function expression
                let func_type = self.check_expression(func_expr)?;
                let func = self.functions.get(&func_type.to_string())
                    .ok_or(TypeError::UndefinedFunction(func_type.to_string()))?;

                // Ensure the number of arguments matches the function's parameter count
                if args.len() != func.params.len() {
                    return Err(TypeError::ArgumentMismatch {
                        function: func.name.clone(),
                        expected: func.params.values().cloned().collect(),
                        found: args.iter().map(|arg| self.check_expression(arg)).collect::<Result<_, _>>()?,
                    });
                }

                // Check each argument against the corresponding parameter type
                for (param, expected_type) in func.params.iter() {
                    let arg_type = self.check_expression(args.iter().find(|&&ref arg| arg.to_string() == param).unwrap())?;
                    if arg_type != *expected_type {
                        return Err(TypeError::TypeMismatch { expected: expected_type.clone(), found: arg_type });
                    }
                }

                // Return the function's return type
                Ok(func.return_type.clone())
            },
        }
    }

    // Checks a function definition for correct parameter types
    fn check_function(&self, func: &Function) -> Result<(), TypeError> {
        // Ensure each parameter is defined and has the correct type
        for (param, param_type) in &func.params {
            if !self.variables.contains_key(param) {
                return Err(TypeError::UndefinedVariable(param.clone()));
            }

            let var_type = self.variables.get(param).unwrap();
            if *var_type != *param_type {
                return Err(TypeError::TypeMismatch { expected: param_type.clone(), found: var_type.clone() });
            }
        }
        Ok(())
    }
}

impl Type {
    // Converts a Type to a string representation for function matching
    fn to_string(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::String => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Function(param_type, return_type) => format!("Function({}, {})", param_type.to_string(), return_type.to_string()),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut type_checker = TypeChecker::new();

    // Define some variables with their types
    type_checker.variables.insert("x".to_string(), Type::Int);
    type_checker.variables.insert("y".to_string(), Type::Float);

    // Define a function with parameters and a return type
    let func = Function {
        name: "add".to_string(),
        params: [("a".to_string(), Type::Int), ("b".to_string(), Type::Int)]
            .iter().cloned().collect(),
        return_type: Type::Int,
    };

    // Insert the function definition into the type checker
    type_checker.functions.insert(func.name.clone(), func);

    // Create an expression representing a binary operation
    let expr = Expression::BinaryOp(
        Box::new(Expression::Variable("x".to_string())),
        "+".to_string(),
        Box::new(Expression::Variable("y".to_string()))
    );

    // Check the type of the binary operation expression
    match type_checker.check_expression(&expr) {
        Ok(expr_type) => println!("Expression type: {:?}", expr_type),
        Err(err) => eprintln!("Error: {}", err),
    }

    // Define a function call with arguments
    let func_call = Expression::FunctionCall(
        Box::new(Expression::Variable("add".to_string())),
        vec![
            Expression::Literal(Type::Int),
            Expression::Literal(Type::Int),
        ]
    );

    // Check the type of the function call expression
    match type_checker.check_expression(&func_call) {
        Ok(call_type) => println!("Function call returns: {:?}", call_type),
        Err(err) => eprintln!("Error: {}", err),
    }

    Ok(())
}