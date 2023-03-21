use std::error::Error;

use nom_json_parser::parse;

fn main() -> Result<(), Box<dyn Error>> {
    let data = "  { \"a\"\t: 42,
    \"b\": [ \"x\", \"y\", 12 ] ,
    \"c\": { \"hello\" : \"world\"
    }
    } ";
    println!(
        "will try to parse valid JSON data:\n\n****************\n{}\n****************\n",
        data
    );

    println!("parsing a valid file:\n{:#?}\n", parse(data)?);
    let data = "  { \"a\"\t: 42,
    \"b\": [ \"x\", \"y\", 12 ] ,
    \"c\": { 1\"hello\" : \"world\"
    }
    } ";

    println!(
        "will try to parse invalid JSON data:\n\n****************\n{}\n****************\n",
        data
    );

    println!("Error information:\n{}\n", parse(data).unwrap_err());
    Ok(())
}
