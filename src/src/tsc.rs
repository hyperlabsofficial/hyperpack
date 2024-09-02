use regex::Regex;
use std::collections::HashMap;

// Function to create the regex pattern for removing TypeScript type annotations
fn create_type_annotation_regex() -> Regex {
    Regex::new(r"(?s)
        (?:\b(?:number|string|boolean|any|void|undefined|object|Array|Record|Tuple|Function|Promise|Set|Map|WeakMap|WeakSet|Symbol|Date|RegExp)\s*<[^>]*>)|
        \b(?:number|string|boolean|any|void|undefined|object|Array|Record|Tuple|Function|Promise|Set|Map|WeakMap|WeakSet|Symbol|Date|RegExp)\s*[\w]+\s*:\s*[^;\n]*?|
        \b(?:interface|type)\s+\w+\s*{[^}]*}|
        \b(?:interface|type)\s+\w+\s*=\s*[^;]*|
        \b(?:const|let|var)\s+[\w]+\s*:\s*[^;\n]*?;\s*|
        \b(?:const|let|var)\s+[\w]+\s*:\s*[^;\n]*?\s*=\s*[^;\n]*?;\s*|
        \b(?:const|let|var)\s+[\w]+\s*=\s*[^;\n]*?;\s*|
        \b(?:function|const|let|var)\s+[\w]+\s*\([^)]*\)\s*:\s*[^;\n]*?|
        \b(?:function|const|let|var)\s+[\w]+\s*=\s*\([^)]*\)\s*:\s*[^;\n]*?|
        \b(?:constructor|new)\s*\([^)]*\)\s*:\s*[^;\n]*?|
        \s*as\s+[^;\n]*|
        <[^>]*>|
        /\*\*[\s\S]*?\*/|
        \b(?:is)\s+[^;\n]*|
        \benum\s+\w+\s*{[^}]*}|
        \bnamespace\s+\w+\s*{[^}]*}|
        @\w+|
        \([^)]*\)\s*:\s*[^;\n]*|
        {[^}]*?:\s*[^;\n]*}|
        \b(?:function|const|let|var)\s+[\w]+\s*<[^>]*>|
        \b(?:public|private|protected)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:abstract|readonly|static)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:keyof|typeof)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:import|export)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:declare|type)\s+\w+\s*=\s*[^;\n]*|
        \b(?:import|export)\s+[\w]+\s*=\s*[^;\n]*|
        \b(?:namespace|module)\s+\w+\s*{[^}]*}|
        \b(?:default|named)\s+\w+\s*=\s*[^;\n]*|
        \b(?:const|let|var)\s+[\w]+\s*=\s*[^;\n]*?;\s*|
        \b(?:function|const|let|var)\s+[\w]+\s*=\s*[^;\n]*?;\s*|
        \b(?:typeof|keyof)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:type|interface)\s+\w+\s*=\s*[^;]*|
        \b(?:abstract|protected|private|public|readonly|static)\s*[\w]+\s*:\s*[^;\n]*|
        \b(?:new)\s*\([^)]*\)\s*:\s*[^;\n]*|
        \b(?:typeof|keyof)\s+[\w]+\s*:\s*[^;\n]*|
        \b(?:interface|type)\s+[\w]+\s*:\s*[^;\n]*"
    ).unwrap()
}

// Function to remove TypeScript type annotations from JavaScript code
fn remove_type_annotations(js_code: &str) -> String {
    let re = create_type_annotation_regex();
    re.replace_all(js_code, "").to_string()
}

// Function to print a test case result
fn print_test_case_result(input: &str, expected: &str, result: &str) {
    if result != expected {
        println!("Test failed:");
        println!("Input: {}", input);
        println!("Expected: {}", expected);
        println!("Got: {}", result);
    } else {
        println!("Test passed.");
    }
}

// Function to test the removal of type annotations with different scenarios
fn test_remove_type_annotations() {
    let cases = vec![
        (
            "type User = { id: number; name: string; age?: number; };",
            "    { id: ; name: ; age?: ; };"
        ),
        (
            "interface Product { name: string; price: number; }",
            "    { name: ; price: ; }"
        ),
        (
            "const getUser = (id: number): Promise<User> => { /* implementation */ };",
            "const getUser = (id: ): => { /* implementation */ };"
        ),
        (
            "function fetchProduct(): Promise<Product> { /* implementation */ }",
            "function fetchProduct() { /* implementation */ }"
        ),
        (
            "const isAvailable = (product: Product): boolean => { /* implementation */ };",
            "const isAvailable = (product: ): => { /* implementation */ };"
        ),
        (
            "const user: User = { id: 1, name: 'John', age: 30 };",
            "const user:  = { id: 1, name: 'John', age: 30 };"
        ),
        (
            "export default User;",
            "export default ;"
        ),
        (
            "type Person = { name: string; age: number; }; function greet(person: Person): void { console.log(person.name); }",
            "    { name: ; age: ; }; function greet(person: ): void { console.log(person.name); }"
        ),
        (
            "namespace MyNamespace { export interface MyInterface { id: number; } }",
            "namespace MyNamespace { export interface MyInterface { id: ; } }"
        ),
        (
            "const myConst: number = 42; let myVar: string = 'hello';",
            "const myConst:  = 42; let myVar:  = 'hello';"
        ),
        (
            "class MyClass { private id: number; constructor(id: number) { this.id = id; } }",
            "class MyClass { private id: ; constructor(id: ) { this.id = id; } }"
        ),
        (
            "interface A extends B { prop: string; }",
            "interface A extends B { prop: ; }"
        ),
        (
            "type Complex = { foo: number; bar: { baz: string; }; };",
            "    { foo: ; bar: { baz: ; }; };"
        ),
        (
            "function process<T>(input: T): T { return input; }",
            "function process(input: ): { return input; }"
        ),
        (
            "const myPromise: Promise<string> = new Promise(resolve => resolve('value'));",
            "const myPromise:  = new Promise(resolve => resolve('value'));"
        ),
        (
            "type LiteralType = 'foo' | 'bar';",
            "    ;"
        ),
        (
            "namespace Utils { export function helper(arg: number): void { /* implementation */ } }",
            "namespace Utils { export function helper(arg: ): void { /* implementation */ } }"
        ),
        (
            "type ComplexType = { a: string; b: number[]; c: { d: boolean; } };",
            "    { a: ; b: []; c: { d: ; } };"
        ),
        (
            "const result: { success: boolean; data: string; } = { success: true, data: 'example' };",
            "const result: { success: ; data: ; } = { success: true, data: 'example' };"
        ),
        // Additional edge cases
        (
            "function genericFunction<T, U>(param1: T, param2: U): T { return param1; }",
            "function genericFunction(param1: , param2: ): { return param1; }"
        ),
        (
            "type UnionType = 'foo' | 'bar' | 42;",
            "    ;"
        ),
        (
            "type FunctionType = (x: number, y: string) => boolean;",
            "    (x: , y: ) => ;"
        ),
        (
            "interface Nested { outer: { inner: { key: string; }; }; };",
            "    { outer: { inner: { key: ; }; }; };"
        ),
        (
            "export class MyExportedClass { static value: number = 10; }",
            "export class MyExportedClass { static value: = 10; }"
        ),
        (
            "const mySet: Set<number> = new Set([1, 2, 3]);",
            "const mySet:  = new Set([1, 2, 3]);"
        ),
        (
            "const myMap: Map<string, number> = new Map();",
            "const myMap:  = new Map();"
        ),
        (
            "class BaseClass { protected baseValue: string; constructor(value: string) { this.baseValue = value; } }",
            "class BaseClass { protected baseValue: ; constructor(value: ) { this.baseValue = value; } }"
        ),
    ];

    for (input, expected) in cases.iter() {
        let result = remove_type_annotations(input);
        print_test_case_result(input, expected, &result);
    }    
}

fn main() {
    test_remove_type_annotations();
}