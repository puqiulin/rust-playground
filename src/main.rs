use json_parser::parser::parse_json;

mod json_parser;

fn main() {
    let json = r#"
    {
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "grades": [85, 90, 92],
        "address": {
            "street": "123 Main St",
            "city": "Anytown"
        }
    }
    "#;

    match parse_json(json) {
        Ok(value) => println!("Parsed JSON: {:?}", value),
        Err(e) => println!("Error parsing JSON: {}", e),
    }
}
